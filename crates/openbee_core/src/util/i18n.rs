//! Internationalization (i18n) system for multi-language support.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data for a single locale containing all translated strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleData {
    pub language_code: String,
    pub language_name: String,
    pub strings: HashMap<String, String>,
}

/// Manages translations and locale switching.
pub struct I18nManager {
    pub current_locale: String,
    pub fallback_locale: String,
    pub locales: HashMap<String, LocaleData>,
    pub available_languages: Vec<(String, String)>,
}

impl I18nManager {
    pub fn new(default_locale: &str) -> Self {
        let mut mgr = Self {
            current_locale: default_locale.to_string(),
            fallback_locale: "en".to_string(),
            locales: HashMap::new(),
            available_languages: Vec::new(),
        };
        mgr.load_locale(Self::default_english_strings());
        mgr.load_locale(Self::default_chinese_strings());
        mgr
    }

    pub fn load_locale(&mut self, data: LocaleData) {
        let entry = (data.language_code.clone(), data.language_name.clone());
        if !self
            .available_languages
            .iter()
            .any(|(c, _)| c == &data.language_code)
        {
            self.available_languages.push(entry);
        }
        self.locales.insert(data.language_code.clone(), data);
    }

    pub fn load_from_json(&mut self, locale: &str, json: &str) -> Result<()> {
        let strings: HashMap<String, String> = serde_json::from_str(json)?;
        if let Some(data) = self.locales.get_mut(locale) {
            data.strings.extend(strings);
        } else {
            self.load_locale(LocaleData {
                language_code: locale.to_string(),
                language_name: locale.to_string(),
                strings,
            });
        }
        Ok(())
    }

    pub fn set_locale(&mut self, locale: &str) -> bool {
        if self.locales.contains_key(locale) {
            self.current_locale = locale.to_string();
            true
        } else {
            false
        }
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(locale) = self.locales.get(&self.current_locale) {
            if let Some(text) = locale.strings.get(key) {
                return text;
            }
        }
        if let Some(fallback) = self.locales.get(&self.fallback_locale) {
            if let Some(text) = fallback.strings.get(key) {
                return text;
            }
        }
        key
    }

    pub fn get_with_args(&self, key: &str, args: &HashMap<String, String>) -> String {
        let mut text = self.get(key).to_string();
        for (k, v) in args {
            text = text.replace(&format!("{{{}}}", k), v);
        }
        text
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.locales
            .get(&self.current_locale)
            .map(|l| l.strings.contains_key(key))
            .unwrap_or(false)
    }

    pub fn available_locales(&self) -> &[(String, String)] {
        &self.available_languages
    }

    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    /// Default English strings for the game.
    pub fn default_english_strings() -> LocaleData {
        let mut s = HashMap::new();
        // Menu
        s.insert("menu.new_game".into(), "New Game".into());
        s.insert("menu.continue".into(), "Continue".into());
        s.insert("menu.options".into(), "Options".into());
        s.insert("menu.quit".into(), "Quit".into());
        s.insert("menu.pause".into(), "Pause".into());
        s.insert("menu.resume".into(), "Resume".into());
        s.insert("menu.save_game".into(), "Save Game".into());
        s.insert("menu.load_game".into(), "Load Game".into());
        s.insert("menu.main_menu".into(), "Main Menu".into());
        s.insert("menu.restart_level".into(), "Restart Level".into());
        // HUD
        s.insert("hud.score".into(), "Score".into());
        s.insert("hud.lives".into(), "Lives".into());
        s.insert("hud.health".into(), "Health".into());
        s.insert("hud.ammo".into(), "Ammo".into());
        s.insert("hud.pistol".into(), "Pistol".into());
        s.insert("hud.dynamite".into(), "Dynamite".into());
        s.insert("hud.magic_claw".into(), "Magic Claw".into());
        s.insert("hud.level".into(), "Level".into());
        s.insert("hud.time".into(), "Time".into());
        // Game states
        s.insert("game.game_over".into(), "Game Over".into());
        s.insert("game.victory".into(), "Victory!".into());
        s.insert("game.level_complete".into(), "Level Complete!".into());
        s.insert("game.boss_fight".into(), "Boss Fight!".into());
        s.insert("game.loading".into(), "Loading...".into());
        s.insert("game.ready".into(), "Ready?".into());
        s.insert("game.go".into(), "GO!".into());
        // Settings
        s.insert("settings.video".into(), "Video".into());
        s.insert("settings.audio".into(), "Audio".into());
        s.insert("settings.controls".into(), "Controls".into());
        s.insert("settings.language".into(), "Language".into());
        s.insert("settings.difficulty".into(), "Difficulty".into());
        s.insert("settings.easy".into(), "Easy".into());
        s.insert("settings.normal".into(), "Normal".into());
        s.insert("settings.hard".into(), "Hard".into());
        s.insert("settings.fullscreen".into(), "Fullscreen".into());
        s.insert("settings.windowed".into(), "Windowed".into());
        s.insert("settings.vsync".into(), "VSync".into());
        s.insert("settings.master_volume".into(), "Master Volume".into());
        s.insert("settings.music_volume".into(), "Music Volume".into());
        s.insert("settings.sfx_volume".into(), "SFX Volume".into());
        s.insert("settings.accessibility".into(), "Accessibility".into());
        s.insert("settings.colorblind".into(), "Colorblind Mode".into());
        // Pickups
        s.insert("pickup.treasure".into(), "Treasure".into());
        s.insert("pickup.health".into(), "Health".into());
        s.insert("pickup.extra_life".into(), "Extra Life!".into());
        s.insert("pickup.map_piece".into(), "Map Piece".into());
        s.insert("pickup.gem".into(), "Gem".into());
        // Achievements
        s.insert(
            "achievement.unlocked".into(),
            "Achievement Unlocked!".into(),
        );
        s.insert("achievement.first_blood".into(), "First Blood".into());
        s.insert(
            "achievement.first_blood_desc".into(),
            "Defeat your first enemy".into(),
        );
        s.insert("achievement.boss_slayer".into(), "Boss Slayer".into());
        s.insert(
            "achievement.boss_slayer_desc".into(),
            "Defeat a boss".into(),
        );
        s.insert(
            "achievement.treasure_hunter".into(),
            "Treasure Hunter".into(),
        );
        s.insert(
            "achievement.treasure_hunter_desc".into(),
            "Collect 1000 treasures".into(),
        );
        s.insert("achievement.speed_demon".into(), "Speed Demon".into());
        s.insert(
            "achievement.speed_demon_desc".into(),
            "Complete a level under 2 minutes".into(),
        );
        s.insert("achievement.untouchable".into(), "Untouchable".into());
        s.insert(
            "achievement.untouchable_desc".into(),
            "Complete a level without taking damage".into(),
        );
        // Speedrun
        s.insert("speedrun.timer".into(), "Timer".into());
        s.insert("speedrun.split".into(), "Split".into());
        s.insert("speedrun.personal_best".into(), "Personal Best".into());
        s.insert("speedrun.new_record".into(), "New Record!".into());
        // Editor
        s.insert("editor.title".into(), "Level Editor".into());
        s.insert("editor.file".into(), "File".into());
        s.insert("editor.edit".into(), "Edit".into());
        s.insert("editor.view".into(), "View".into());
        s.insert("editor.new_level".into(), "New Level".into());
        s.insert("editor.save".into(), "Save".into());
        s.insert("editor.load".into(), "Load".into());
        s.insert("editor.undo".into(), "Undo".into());
        s.insert("editor.redo".into(), "Redo".into());
        // Multiplayer
        s.insert("net.connecting".into(), "Connecting...".into());
        s.insert("net.connected".into(), "Connected".into());
        s.insert("net.disconnected".into(), "Disconnected".into());
        s.insert("net.lobby".into(), "Lobby".into());
        s.insert("net.ready".into(), "Ready".into());
        s.insert("net.start".into(), "Start Game".into());
        s.insert("net.chat".into(), "Chat".into());

        LocaleData {
            language_code: "en".into(),
            language_name: "English".into(),
            strings: s,
        }
    }

    /// Default Chinese strings for the game.
    pub fn default_chinese_strings() -> LocaleData {
        let mut s = HashMap::new();
        s.insert("menu.new_game".into(), "新游戏".into());
        s.insert("menu.continue".into(), "继续".into());
        s.insert("menu.options".into(), "选项".into());
        s.insert("menu.quit".into(), "退出".into());
        s.insert("menu.pause".into(), "暂停".into());
        s.insert("menu.resume".into(), "继续游戏".into());
        s.insert("menu.save_game".into(), "保存游戏".into());
        s.insert("menu.load_game".into(), "加载游戏".into());
        s.insert("menu.main_menu".into(), "主菜单".into());
        s.insert("menu.restart_level".into(), "重新开始关卡".into());
        s.insert("hud.score".into(), "分数".into());
        s.insert("hud.lives".into(), "生命".into());
        s.insert("hud.health".into(), "血量".into());
        s.insert("hud.ammo".into(), "弹药".into());
        s.insert("hud.pistol".into(), "手枪".into());
        s.insert("hud.dynamite".into(), "炸药".into());
        s.insert("hud.magic_claw".into(), "魔法爪".into());
        s.insert("hud.level".into(), "关卡".into());
        s.insert("hud.time".into(), "时间".into());
        s.insert("game.game_over".into(), "游戏结束".into());
        s.insert("game.victory".into(), "胜利！".into());
        s.insert("game.level_complete".into(), "关卡完成！".into());
        s.insert("game.boss_fight".into(), "Boss战！".into());
        s.insert("game.loading".into(), "加载中...".into());
        s.insert("game.ready".into(), "准备好了吗？".into());
        s.insert("game.go".into(), "开始！".into());
        s.insert("settings.video".into(), "视频".into());
        s.insert("settings.audio".into(), "音频".into());
        s.insert("settings.controls".into(), "控制".into());
        s.insert("settings.language".into(), "语言".into());
        s.insert("settings.difficulty".into(), "难度".into());
        s.insert("settings.easy".into(), "简单".into());
        s.insert("settings.normal".into(), "普通".into());
        s.insert("settings.hard".into(), "困难".into());
        s.insert("settings.fullscreen".into(), "全屏".into());
        s.insert("settings.accessibility".into(), "辅助功能".into());
        s.insert("settings.colorblind".into(), "色盲模式".into());
        s.insert("pickup.treasure".into(), "宝藏".into());
        s.insert("pickup.health".into(), "生命值".into());
        s.insert("pickup.extra_life".into(), "额外生命！".into());
        s.insert("achievement.unlocked".into(), "成就解锁！".into());
        s.insert("speedrun.timer".into(), "计时器".into());
        s.insert("speedrun.personal_best".into(), "个人最佳".into());
        s.insert("speedrun.new_record".into(), "新纪录！".into());
        s.insert("editor.title".into(), "关卡编辑器".into());
        s.insert("net.connecting".into(), "连接中...".into());
        s.insert("net.lobby".into(), "大厅".into());
        s.insert("net.ready".into(), "准备".into());
        s.insert("net.start".into(), "开始游戏".into());
        s.insert("net.chat".into(), "聊天".into());

        LocaleData {
            language_code: "zh".into(),
            language_name: "中文".into(),
            strings: s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_basic() {
        let mgr = I18nManager::new("en");
        assert_eq!(mgr.get("menu.new_game"), "New Game");
        assert_eq!(mgr.get("nonexistent_key"), "nonexistent_key");
    }

    #[test]
    fn test_i18n_switch_locale() {
        let mut mgr = I18nManager::new("en");
        assert_eq!(mgr.get("menu.new_game"), "New Game");
        mgr.set_locale("zh");
        assert_eq!(mgr.get("menu.new_game"), "新游戏");
    }

    #[test]
    fn test_i18n_args() {
        let mut mgr = I18nManager::new("en");
        mgr.locales
            .get_mut("en")
            .unwrap()
            .strings
            .insert("greeting".into(), "Hello, {name}! Score: {score}".into());
        let mut args = HashMap::new();
        args.insert("name".into(), "Captain Claw".into());
        args.insert("score".into(), "9999".into());
        assert_eq!(
            mgr.get_with_args("greeting", &args),
            "Hello, Captain Claw! Score: 9999"
        );
    }
}
