//! Persisted project data. Field names are camelCase so the JSON matches the frontend store.

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub paper: String,
    pub pages: Vec<PageData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageData {
    pub caption: String,
    #[serde(default)]
    pub sub_caption: String,
    #[serde(default)]
    pub author: String,
    pub caption_style: CaptionStyle,
    pub images: Vec<SlotData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CaptionStyle {
    pub h: String,
    pub v: String,
    pub size: String,
    pub color: String,
    pub custom: bool,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SlotData {
    pub path: Option<String>,
    pub mode: String,
    pub offset_x: f64,
    pub offset_y: f64,
    pub aspect: String,
}

pub const PAGE_COUNT: usize = 8;

impl Project {
    fn validate(&self) -> Result<(), String> {
        if self.pages.len() != PAGE_COUNT {
            return Err(format!("expected {PAGE_COUNT} pages, found {}", self.pages.len()));
        }
        if !matches!(self.paper.as_str(), "letter" | "a4") {
            return Err(format!("unknown paper size '{}'", self.paper));
        }
        if self.pages.iter().any(|p| p.images.len() != 2) {
            return Err("each page must have exactly two image slots".into());
        }
        Ok(())
    }
}

#[tauri::command]
pub fn save_project(path: String, project: Project) -> Result<(), String> {
    project.validate()?;
    let json = serde_json::to_string_pretty(&project).map_err(|e| e.to_string())?;
    // Write to a temp file first so a crash mid-write can't corrupt an existing project.
    let tmp = format!("{path}.tmp");
    fs::write(&tmp, json).map_err(|e| format!("could not write {tmp}: {e}"))?;
    fs::rename(&tmp, &path).map_err(|e| format!("could not save {path}: {e}"))
}

#[tauri::command]
pub fn load_project(path: String) -> Result<Project, String> {
    let text = fs::read_to_string(&path).map_err(|e| format!("could not read {path}: {e}"))?;
    let project: Project =
        serde_json::from_str(&text).map_err(|e| format!("not a valid project file: {e}"))?;
    project.validate()?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Project {
        let page = |c: &str| PageData {
            caption: c.into(),
            sub_caption: String::new(),
            author: String::new(),
            caption_style: CaptionStyle {
                h: "center".into(),
                v: "bottom".into(),
                size: "md".into(),
                color: "white".into(),
                custom: false,
                x: 50.0,
                y: 85.0,
            },
            images: (0..2)
                .map(|_| SlotData {
                    path: None,
                    mode: "fit".into(),
                    offset_x: 50.0,
                    offset_y: 50.0,
                    aspect: "auto".into(),
                })
                .collect(),
        };
        Project { paper: "a4".into(), pages: (0..PAGE_COUNT).map(|i| page(&i.to_string())).collect() }
    }

    #[test]
    fn save_and_load_round_trip() {
        let path = std::env::temp_dir().join("looseleaf_roundtrip_test.looseleaf");
        let path = path.to_string_lossy().to_string();
        save_project(path.clone(), sample()).unwrap();
        let loaded = load_project(path.clone()).unwrap();
        assert_eq!(loaded.paper, "a4");
        assert_eq!(loaded.pages[3].caption, "3");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_wrong_page_count() {
        let mut p = sample();
        p.pages.pop();
        assert!(p.validate().is_err());
    }
}
