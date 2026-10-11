use serde::Serialize;

use crate::{Config, Gaming, Layout, Rect, Rule, Zone};

/// A display.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Monitor {
    /// Identifies the display across restarts and reconnects.
    pub id: String,
    /// What the display calls itself, such as its model.
    pub name: String,
    pub bounds: Rect,
    /// The bounds minus whatever the desktop reserves, such as a taskbar.
    pub work_area: Rect,
}

/// The properties of a window that a Rule can match.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Window {
    /// File name of the executable that owns the window.
    pub exe: String,
    /// Full path of that executable.
    pub path: String,
    /// Window class, or empty where the platform has none.
    pub class: String,
    pub title: String,
}

impl Config {
    /// Where `window` belongs: the Zone of the first Rule that matches it and whose
    /// Monitor is attached.
    pub fn placement(&self, window: &Window, monitors: &[Monitor]) -> Option<(&Rule, Rect)> {
        self.rules.iter().filter(|rule| rule.matches(window)).find_map(|rule| {
            let monitor = monitors.iter().find(|monitor| monitor.id == rule.monitor)?;
            let layout = self.layouts.iter().find(|layout| layout.name == rule.layout)?;
            let zone = layout.zones.iter().find(|zone| zone.name == rule.zone)?;
            Some((rule, layout.rect(zone, monitor.work_area)))
        })
    }

    /// Whether any Rule names `exe`, whatever else that Rule requires.
    pub fn names(&self, exe: &str) -> bool {
        self.rules.iter().any(|rule| same_file(&rule.exe, exe))
    }

    /// How many Rules cannot apply because their Monitor is not among `monitors`.
    pub fn skipped(&self, monitors: &[Monitor]) -> usize {
        let attached = |rule: &&Rule| monitors.iter().any(|monitor| monitor.id == rule.monitor);
        self.rules.len() - self.rules.iter().filter(attached).count()
    }
}

impl Rule {
    fn matches(&self, window: &Window) -> bool {
        same_file(&self.exe, &window.exe)
            && self.class.as_ref().is_none_or(|class| *class == window.class)
            && self.title_contains.as_ref().is_none_or(|part| window.title.contains(part))
    }
}

impl Gaming {
    /// Whether `exe` is one of the listed games.
    pub fn lists(&self, exe: &str) -> bool {
        self.games.iter().any(|game| same_file(game, exe))
    }
}

impl Layout {
    /// The pixels `zone` covers when this Layout's grid is stretched over `area`.
    ///
    /// Each edge is a grid line, and a grid line has one pixel position however many
    /// Zones meet at it, so neighbouring Zones never gap or overlap.
    pub fn rect(&self, zone: &Zone, area: Rect) -> Rect {
        let x = |column| area.left + grid_line(area.width(), column, self.columns.get());
        let y = |row| area.top + grid_line(area.height(), row, self.rows.get());
        Rect {
            left: x(zone.column),
            top: y(zone.row),
            right: x(zone.column.saturating_add(zone.width.get())),
            bottom: y(zone.row.saturating_add(zone.height.get())),
        }
    }
}

/// The pixel offset of grid line `index` when `count` cells span `extent` pixels.
fn grid_line(extent: i32, index: u32, count: u32) -> i32 {
    (i64::from(extent) * i64::from(index) / i64::from(count)) as i32
}

/// Whether two file names are the same to a case-insensitive file system.
fn same_file(a: &str, b: &str) -> bool {
    let fold = |name| str::chars(name).flat_map(char::to_lowercase);
    fold(a).eq(fold(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect { left: 0, top: 0, right: 100, bottom: 100 };

    fn monitor(id: &str) -> Monitor {
        Monitor { id: id.into(), name: String::new(), bounds: AREA, work_area: AREA }
    }

    fn window(exe: &str, class: &str, title: &str) -> Window {
        Window { exe: exe.into(), path: String::new(), class: class.into(), title: title.into() }
    }

    /// A config with one Layout, `grid`, whose Zones are its cells, each named
    /// `column,row`. Every entry of `rules` is the body of a `[[rule]]` that uses it.
    fn config(columns: u32, rows: u32, rules: &[&str]) -> Config {
        let cells = (0..columns).flat_map(|column| (0..rows).map(move |row| (column, row)));
        let zones: String = cells
            .map(|(column, row)| {
                format!("{{ name = '{column},{row}', column = {column}, row = {row}, width = 1, height = 1 }},")
            })
            .collect();
        let rules: String =
            rules.iter().map(|rule| format!("[[rule]]\nlayout = 'grid'\n{rule}\n")).collect();
        format!(
            "version = 1\n[[layout]]\nname = 'grid'\ncolumns = {columns}\nrows = {rows}\nzones = [{zones}]\n{rules}"
        )
        .parse()
        .unwrap()
    }

    #[test]
    fn zones_tile_any_work_area_without_gaps_or_overlaps() {
        for (columns, rows) in [(2, 1), (3, 1), (4, 1), (2, 2), (3, 2), (7, 5)] {
            let config = config(columns, rows, &[]);
            let layout = &config.layouts[0];
            for (width, height) in [(1920, 1080), (2560, 1400), (1921, 1079), (1366, 767), (5, 3)] {
                let area = Rect { left: -width, top: 40, right: 0, bottom: 40 + height };
                let rects: Vec<Rect> =
                    layout.zones.iter().map(|zone| layout.rect(zone, area)).collect();

                // Disjoint rectangles inside the area that add up to its size cover it exactly.
                let pixels: i32 = rects.iter().map(|rect| rect.width() * rect.height()).sum();
                assert_eq!(pixels, width * height, "{columns}x{rows} over {width}x{height}");
                for (index, a) in rects.iter().enumerate() {
                    assert!(area.left <= a.left && a.right <= area.right, "{a:?}");
                    assert!(area.top <= a.top && a.bottom <= area.bottom, "{a:?}");
                    for b in &rects[index + 1..] {
                        let beside = a.right <= b.left || b.right <= a.left;
                        let stacked = a.bottom <= b.top || b.bottom <= a.top;
                        assert!(beside || stacked, "{a:?} overlaps {b:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn thirds_of_an_odd_width_differ_by_at_most_a_pixel() {
        let config = config(3, 1, &[]);
        let layout = &config.layouts[0];
        let area = Rect { left: 0, top: 0, right: 1921, bottom: 1080 };
        let widths: Vec<i32> =
            layout.zones.iter().map(|zone| layout.rect(zone, area).width()).collect();
        assert_eq!(widths, [640, 640, 641]);
    }

    #[test]
    fn a_zone_may_span_cells() {
        let mut config = config(3, 2, &[]);
        let layout = &mut config.layouts[0];
        layout.zones[0].width = 2.try_into().unwrap();
        layout.zones[0].height = 2.try_into().unwrap();
        let area = Rect { left: 0, top: 0, right: 300, bottom: 200 };
        let expected = Rect { left: 0, top: 0, right: 200, bottom: 200 };
        assert_eq!(layout.rect(&layout.zones[0], area), expected);
    }

    #[test]
    fn exe_matching_ignores_case() {
        let config = config(2, 1, &["exe = 'Discord.EXE'\nmonitor = 'desk'\nzone = '0,0'"]);
        let monitors = [monitor("desk")];
        assert!(config.placement(&window("discord.exe", "", ""), &monitors).is_some());
        assert!(config.placement(&window("DISCORD.exe", "", ""), &monitors).is_some());
        assert!(config.placement(&window("discord.exe.bak", "", ""), &monitors).is_none());
        assert!(config.names("dIsCoRd.ExE") && !config.names("slack.exe"));
    }

    #[test]
    fn the_first_matching_rule_wins() {
        let config = config(
            4,
            1,
            &[
                "exe = 'code.exe'\ntitle_contains = 'notes'\nmonitor = 'desk'\nzone = '0,0'",
                "exe = 'code.exe'\nclass = 'Popup'\nmonitor = 'desk'\nzone = '1,0'",
                "exe = 'code.exe'\nmonitor = 'desk'\nzone = '2,0'",
                "exe = 'code.exe'\nmonitor = 'desk'\nzone = '3,0'",
            ],
        );
        let monitors = [monitor("desk")];
        let zone = |window| config.placement(&window, &monitors).unwrap().0.zone.as_str();
        assert_eq!(zone(window("code.exe", "Chrome_WidgetWin_1", "my notes - Code")), "0,0");
        assert_eq!(zone(window("code.exe", "Popup", "Open File")), "1,0");
        assert_eq!(zone(window("code.exe", "Chrome_WidgetWin_1", "main.rs - Code")), "2,0");
    }

    #[test]
    fn a_rule_for_a_missing_monitor_is_skipped() {
        let config = config(
            2,
            1,
            &[
                "exe = 'code.exe'\nmonitor = 'docked'\nzone = '0,0'",
                "exe = 'code.exe'\nmonitor = 'laptop'\nzone = '1,0'",
            ],
        );
        let code = window("code.exe", "", "");
        let laptop = [monitor("laptop")];
        let (rule, target) = config.placement(&code, &laptop).unwrap();
        assert_eq!(rule.monitor, "laptop");
        assert_eq!(target, Rect { left: 50, top: 0, right: 100, bottom: 100 });
        assert_eq!(config.skipped(&laptop), 1);
        assert_eq!(config.placement(&code, &[]), None);
    }

    #[test]
    fn games_are_listed_without_regard_to_case() {
        let gaming = Gaming { games: vec!["EldenRing.exe".into()], ..Gaming::default() };
        assert!(gaming.lists("eldenring.exe") && !gaming.lists("eldenring"));
    }
}
