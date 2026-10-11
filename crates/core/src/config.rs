use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Everything the user can configure, as stored in `config.toml`.
///
/// Parse one with [`str::parse`] and write one with [`ToString::to_string`]. A parsed
/// `Config` is valid; one built or edited in memory is not until [`Config::validate`]
/// has passed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    version: Version,
    #[serde(default)]
    pub gaming: Gaming,
    #[serde(default, rename = "monitor")]
    pub monitors: Vec<MonitorName>,
    #[serde(default, rename = "layout")]
    pub layouts: Vec<Layout>,
    /// In priority order: the first Rule that matches a window claims it.
    #[serde(default, rename = "rule")]
    pub rules: Vec<Rule>,
}

/// When Gaming mode starts on its own.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Gaming {
    pub enabled: bool,
    /// Treat any app running exclusive-fullscreen Direct3D as a game.
    pub detect_fullscreen: bool,
    /// Executable file names, compared without regard to case.
    pub games: Vec<String>,
}

/// The user's own name for a Monitor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorName {
    pub id: String,
    pub name: String,
}

/// A named grid over a Monitor's work area, and the Zones cut from it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub name: String,
    pub columns: NonZeroU32,
    pub rows: NonZeroU32,
    pub zones: Vec<Zone>,
}

/// A named rectangle of grid cells within a Layout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub name: String,
    pub column: u32,
    pub row: u32,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

/// Which windows to move, and where to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Executable file name, compared without regard to case.
    pub exe: String,
    /// Exact window class, if the Rule is limited to one.
    pub class: Option<String>,
    /// Text the window title must contain, if the Rule is limited by title.
    pub title_contains: Option<String>,
    /// The Monitor's id.
    pub monitor: String,
    pub layout: String,
    pub zone: String,
    /// Place the window a second time after this long, for apps that resize themselves
    /// once they have loaded. Zero places it once.
    #[serde(default)]
    pub reapply_after_ms: u32,
}

/// Why a config was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    Syntax { line: usize, message: String },
    DuplicateLayout { layout: String },
    DuplicateZone { layout: String, zone: String },
    ZoneOutsideGrid { layout: String, zone: String },
    UnknownLayout { exe: String, layout: String },
    UnknownZone { exe: String, layout: String, zone: String },
}

impl Config {
    /// Checks what the types cannot express: that names are unique, Zones fit their
    /// grid, and every Rule refers to a Zone that exists.
    pub fn validate(&self) -> Result<(), ConfigError> {
        for (index, layout) in self.layouts.iter().enumerate() {
            let name = || layout.name.clone();
            if self.layouts[..index].iter().any(|earlier| earlier.name == layout.name) {
                return Err(ConfigError::DuplicateLayout { layout: name() });
            }
            for (index, zone) in layout.zones.iter().enumerate() {
                let zone_name = || zone.name.clone();
                if layout.zones[..index].iter().any(|earlier| earlier.name == zone.name) {
                    return Err(ConfigError::DuplicateZone { layout: name(), zone: zone_name() });
                }
                if !layout.contains(zone) {
                    return Err(ConfigError::ZoneOutsideGrid { layout: name(), zone: zone_name() });
                }
            }
        }
        for rule in &self.rules {
            let (exe, layout) = (rule.exe.clone(), rule.layout.clone());
            let Some(found) = self.layouts.iter().find(|layout| layout.name == rule.layout) else {
                return Err(ConfigError::UnknownLayout { exe, layout });
            };
            if !found.zones.iter().any(|zone| zone.name == rule.zone) {
                return Err(ConfigError::UnknownZone { exe, layout, zone: rule.zone.clone() });
            }
        }
        Ok(())
    }
}

impl Layout {
    fn contains(&self, zone: &Zone) -> bool {
        let fits = |start: u32, length: NonZeroU32, cells: NonZeroU32| {
            start.checked_add(length.get()).is_some_and(|end| end <= cells.get())
        };
        fits(zone.column, zone.width, self.columns) && fits(zone.row, zone.height, self.rows)
    }
}

impl FromStr for Config {
    type Err = ConfigError;

    fn from_str(text: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(text).map_err(|error| ConfigError::Syntax {
            line: error.span().map_or(1, |span| 1 + text[..span.start].matches('\n').count()),
            message: error.message().to_owned(),
        })?;
        config.validate()?;
        Ok(config)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&toml::to_string(self).map_err(|_| fmt::Error)?)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { line, message } => write!(f, "line {line}: {message}"),
            Self::DuplicateLayout { layout } => write!(f, "layout {layout:?} is defined twice"),
            Self::DuplicateZone { layout, zone } => {
                write!(f, "zone {zone:?} is defined twice in layout {layout:?}")
            }
            Self::ZoneOutsideGrid { layout, zone } => {
                write!(f, "zone {zone:?} does not fit the grid of layout {layout:?}")
            }
            Self::UnknownLayout { exe, layout } => {
                write!(f, "the rule for {exe} uses layout {layout:?}, which does not exist")
            }
            Self::UnknownZone { exe, layout, zone } => write!(
                f,
                "the rule for {exe} uses zone {zone:?}, which layout {layout:?} does not have"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// The config format's version number. No other number deserialises, so a `Config`
/// that exists is one this build understands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Version;

impl Version {
    const NUMBER: u32 = 1;
}

impl Serialize for Version {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(Self::NUMBER)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match u32::deserialize(deserializer)? {
            Self::NUMBER => Ok(Self),
            other => Err(D::Error::custom(format_args!("version {other} is not supported"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THIRDS: &str = r#"
        version = 1

        [gaming]
        enabled = true
        games = ["eldenring.exe"]

        [[monitor]]
        id = "DEL-A0EC"
        name = "Left"

        [[layout]]
        name = "thirds"
        columns = 3
        rows = 1
        zones = [
            { name = "left",   column = 0, row = 0, width = 1, height = 1 },
            { name = "middle", column = 1, row = 0, width = 1, height = 1 },
            { name = "right",  column = 2, row = 0, width = 1, height = 1 },
        ]

        [[rule]]
        exe = "discord.exe"
        title_contains = "Discord"
        monitor = "DEL-A0EC"
        layout = "thirds"
        zone = "right"
        reapply_after_ms = 500
    "#;

    fn rejection(text: &str) -> ConfigError {
        text.parse::<Config>().expect_err("the config should be rejected")
    }

    #[test]
    fn parses_every_section() {
        let config: Config = THIRDS.parse().unwrap();
        assert!(config.gaming.enabled && !config.gaming.detect_fullscreen);
        assert_eq!(config.gaming.games, ["eldenring.exe"]);
        assert_eq!(config.monitors[0].name, "Left");
        assert_eq!(config.layouts[0].zones.len(), 3);
        assert_eq!(config.rules[0].class, None);
        assert_eq!(config.rules[0].reapply_after_ms, 500);
    }

    #[test]
    fn survives_a_round_trip_through_its_own_output() {
        let config: Config = THIRDS.parse().unwrap();
        assert_eq!(config.to_string().parse(), Ok(config));
    }

    #[test]
    fn needs_nothing_but_a_version() {
        assert_eq!("version = 1".parse(), Ok(Config::default()));
    }

    #[test]
    fn rejects_malformed_input_without_panicking() {
        let malformed =
            ["", "version", "version = 1\n[[rule]]\nexe = 7", "\0\u{feff}[[[", "rule = 1"];
        for text in malformed {
            assert!(matches!(rejection(text), ConfigError::Syntax { .. }), "{text:?}");
        }
    }

    #[test]
    fn reports_the_line_an_error_is_on() {
        let error = rejection("version = 1\n\n[gaming]\nenabled = 3");
        assert!(matches!(error, ConfigError::Syntax { line: 4, .. }), "{error}");
    }

    #[test]
    fn rejects_versions_and_keys_it_does_not_know() {
        assert_eq!(rejection("version = 2").to_string(), "line 1: version 2 is not supported");
        let error = rejection("version = 1\ncolour = 1");
        assert!(matches!(error, ConfigError::Syntax { line: 2, .. }), "{error}");
    }

    #[test]
    fn rejects_a_grid_without_cells() {
        let error = rejection(&THIRDS.replace("columns = 3", "columns = 0"));
        assert!(matches!(error, ConfigError::Syntax { .. }), "{error}");
    }

    #[test]
    fn rejects_duplicate_names() {
        let again = "[[layout]]\nname = \"thirds\"\ncolumns = 1\nrows = 1\nzones = []";
        let error = rejection(&format!("{THIRDS}\n{again}"));
        assert_eq!(error, ConfigError::DuplicateLayout { layout: "thirds".into() });

        let error = rejection(&THIRDS.replace("\"middle\"", "\"left\""));
        let expected = ConfigError::DuplicateZone { layout: "thirds".into(), zone: "left".into() };
        assert_eq!(error, expected);
    }

    #[test]
    fn rejects_a_zone_that_leaves_the_grid() {
        for column in ["column = 3", "column = 4294967295"] {
            let error = rejection(&THIRDS.replace("column = 2", column));
            let expected =
                ConfigError::ZoneOutsideGrid { layout: "thirds".into(), zone: "right".into() };
            assert_eq!(error, expected);
        }
    }

    #[test]
    fn rejects_dangling_references() {
        let error = rejection(&THIRDS.replace("layout = \"thirds\"", "layout = \"halves\""));
        let expected =
            ConfigError::UnknownLayout { exe: "discord.exe".into(), layout: "halves".into() };
        assert_eq!(error, expected);

        let error = rejection(&THIRDS.replace("zone = \"right\"", "zone = \"centre\""));
        assert_eq!(
            error.to_string(),
            "the rule for discord.exe uses zone \"centre\", which layout \"thirds\" does not have"
        );
    }
}
