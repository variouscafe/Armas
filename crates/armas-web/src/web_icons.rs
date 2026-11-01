//! Web-specific icons (parsed from embedded SVGs at runtime)

use armas::icon::OwnedIconData;
use std::sync::OnceLock;

const DARK_SVG: &str = include_str!("../assets/icons/dark.svg");
const LIGHT_SVG: &str = include_str!("../assets/icons/light.svg");
const SYSTEM_SVG: &str = include_str!("../assets/icons/system.svg");

static DARK: OnceLock<OwnedIconData> = OnceLock::new();
static LIGHT: OnceLock<OwnedIconData> = OnceLock::new();
static SYSTEM: OnceLock<OwnedIconData> = OnceLock::new();

pub fn dark() -> &'static OwnedIconData {
    DARK.get_or_init(|| armas_icon::runtime::parse_svg_named(DARK_SVG, "dark").unwrap())
}

pub fn light() -> &'static OwnedIconData {
    LIGHT.get_or_init(|| armas_icon::runtime::parse_svg_named(LIGHT_SVG, "light").unwrap())
}

pub fn system() -> &'static OwnedIconData {
    SYSTEM.get_or_init(|| armas_icon::runtime::parse_svg_named(SYSTEM_SVG, "system").unwrap())
}

#[cfg(test)]
mod icon_tests {
    /// The icons parse lazily at first render with an unwrap — prove the
    /// embedded SVGs are valid here, in a test, instead of at a user's
    /// first paint.
    #[test]
    fn embedded_svgs_parse() {
        let _ = super::dark();
        let _ = super::light();
        let _ = super::system();
    }
}
