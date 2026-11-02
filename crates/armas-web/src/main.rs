#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod components;
mod markdown;
mod showcase_gen;
pub mod syntax;
mod url_utils;
pub mod web_icons;

use armas::*;
use components::{ComponentsListPage, SiteHeader, SiteHero, SiteSidebar, ThemeToggleIcon};
use eframe::egui;

// =============================================================================
// Constants
// =============================================================================

mod layout {
    pub const SIDEBAR_WIDTH: f32 = 220.0;
    pub const MOBILE_BREAKPOINT: f32 = 768.0;
    pub const CONTENT_MAX_WIDTH: f32 = 768.0;
}

// =============================================================================
// Theme
// =============================================================================

#[derive(Debug, Clone)]
enum AppTheme {
    Dark(Theme),
    Light(Theme),
}

impl AppTheme {
    fn dark() -> Self {
        let mut theme = Theme::dark();
        // shadcn zinc dark
        theme.colors.background = [9, 9, 11];
        theme.colors.card = [9, 9, 11];
        theme.colors.muted = [39, 39, 42];
        theme.colors.border = [39, 39, 42];
        theme.colors.foreground = [250, 250, 250];
        theme.colors.muted_foreground = [161, 161, 170];
        theme.colors.primary = [250, 250, 250];
        theme.colors.primary_foreground = [9, 9, 11];
        theme.colors.accent = [39, 39, 42];
        theme.colors.accent_foreground = [250, 250, 250];
        Self::Dark(theme)
    }

    fn light() -> Self {
        let mut theme = Theme::light();
        // shadcn zinc light
        theme.colors.background = [255, 255, 255];
        theme.colors.card = [255, 255, 255];
        theme.colors.muted = [244, 244, 245];
        theme.colors.border = [228, 228, 231];
        theme.colors.foreground = [9, 9, 11];
        theme.colors.muted_foreground = [113, 113, 122];
        theme.colors.primary = [24, 24, 27];
        theme.colors.primary_foreground = [250, 250, 250];
        theme.colors.accent = [244, 244, 245];
        theme.colors.accent_foreground = [24, 24, 27];
        Self::Light(theme)
    }

    fn theme(&self) -> &Theme {
        match self {
            Self::Dark(t) | Self::Light(t) => t,
        }
    }

    fn is_dark(&self) -> bool {
        matches!(self, Self::Dark(_))
    }
}

// =============================================================================
// App State
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum PageState {
    Hero,
    ComponentsList,
    DocsPage(usize),
}

type PageShowFn = fn(&mut egui::Ui);
type Page = (&'static str, PageShowFn);

struct ShowcaseApp {
    theme: AppTheme,
    /// Last egui mode the system-follow branch has seen. Starts `None` so
    /// the FIRST ui() pass always re-derives the theme from the actual
    /// mode - `cc.egui_ctx.theme()` at `new()` time can resolve System to
    /// the Dark fallback (or, with persistence enabled, to a stale
    /// restored system theme) depending on when the real theme arrives.
    /// Only meaningful while the source is `ThemeSource::System`.
    last_mode: Option<egui::Theme>,
    /// Where the effective theme comes from: the system theme, or a user
    /// pin reached by cycling the header toggle (System → Light → Dark →
    /// System — the toggle icon depicts the NEXT state).
    theme_source: ThemeSource,
    search_text: String,
    page_state: PageState,
    sidebar_open: bool,
    pages: Vec<Page>,
}

/// Source of the effective theme. `System` follows the OS/browser theme
/// (including runtime changes); `Light`/`Dark` are user pins.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeSource {
    System,
    Light,
    Dark,
}

impl ThemeSource {
    fn cycle(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }

    /// The header toggle icon depicts the NEXT state in the cycle.
    fn toggle_icon(self) -> ThemeToggleIcon {
        match self {
            Self::System => ThemeToggleIcon::Light,
            Self::Light => ThemeToggleIcon::Dark,
            Self::Dark => ThemeToggleIcon::System,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

/// eframe storage key for the persisted theme source (a file on native,
/// localStorage on wasm — eframe provides both behind the same API).
const THEME_KEY: &str = "armas_theme_source";

impl ShowcaseApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Restore the persisted theme source, defaulting to System.
        let theme_source = cc
            .storage
            .and_then(|s| s.get_string(THEME_KEY))
            .as_deref()
            .and_then(ThemeSource::parse)
            .unwrap_or(ThemeSource::System);

        let theme = match theme_source {
            ThemeSource::System => {
                // Re-assert the System preference: eframe unconditionally
                // RESTORES egui memory before new(), so a stale pinned
                // preference (from a pre-persist_egui_memory build, or
                // another eframe app on the same origin) would otherwise
                // hijack System mode and swallow every theme-change event.
                cc.egui_ctx.set_theme(egui::ThemePreference::System);
                // Best guess for the field only - the follow branch
                // re-derives from the real mode and styles on the FIRST
                // ui() pass (see `last_mode`), which always precedes the
                // first paint, so no apply_theme here.
                Self::theme_for_mode(cc.egui_ctx.theme())
            }
            source => {
                let theme = if source == ThemeSource::Light {
                    AppTheme::light()
                } else {
                    AppTheme::dark()
                };
                // A restored pin also drives the egui preference so the
                // mode and the tint agree from the first frame.
                cc.egui_ctx.set_theme(Self::preference_for(&theme));
                Self::apply_theme(&theme, &cc.egui_ctx);
                theme
            }
        };

        Self::setup_fonts(&cc.egui_ctx);

        // Parse URL and set initial page
        let page_state = Self::page_from_url();

        Self {
            theme,
            last_mode: None,
            theme_source,
            search_text: String::new(),
            page_state,
            sidebar_open: false,
            pages: showcase_gen::get_pages(),
        }
    }

    /// Parse current URL and return page state
    fn page_from_url() -> PageState {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(hash) = url_utils::url::get_hash() {
                if hash == "components"
                    || hash == "/components"
                    || hash == "basic"
                    || hash == "/basic"
                {
                    return PageState::ComponentsList;
                }
                if let Some((section, component)) = url_utils::url::parse_route(&hash) {
                    if let Some(idx) = showcase_gen::get_page_by_route(&section, &component) {
                        return PageState::DocsPage(idx);
                    }
                }
            }
        }
        PageState::Hero
    }

    fn setup_fonts(ctx: &egui::Context) {
        let mut font_builder = armas::FontFamilyBuilder::new();
        font_builder
            .add_family(
                "Inter",
                include_bytes!("../../armas-basic/fonts/Inter-Regular.otf"),
                Some(include_bytes!("../../armas-basic/fonts/Inter-Medium.otf")),
                Some(include_bytes!("../../armas-basic/fonts/Inter-SemiBold.otf")),
                Some(include_bytes!("../../armas-basic/fonts/Inter-Bold.otf")),
            )
            .set_default("Inter");

        font_builder.add_family(
            "FiraMono",
            include_bytes!("../assets/fonts/FiraMono-Regular.ttf"),
            None,
            None,
            None,
        );

        font_builder.install(ctx, true);
    }

    /// Cycle the theme source: System → Light → Dark → System. Returning to
    /// System resumes following the OS/browser theme (including runtime
    /// changes); the pin states also flip the egui mode so the background
    /// and the foreground colors always land in the same mode.
    fn cycle_theme(&mut self, ctx: &egui::Context) {
        self.theme_source = self.theme_source.cycle();
        match self.theme_source {
            ThemeSource::System => {
                ctx.set_theme(egui::ThemePreference::System);
                let mode = ctx.theme();
                self.last_mode = Some(mode);
                self.theme = Self::theme_for_mode(mode);
            }
            ThemeSource::Light | ThemeSource::Dark => {
                self.theme = if self.theme_source == ThemeSource::Dark {
                    AppTheme::dark()
                } else {
                    AppTheme::light()
                };
                self.last_mode = Some(Self::mode_for(&self.theme));
                ctx.set_theme(Self::preference_for(&self.theme));
            }
        }
        Self::apply_theme(&self.theme, ctx);
    }

    /// Map an egui theme mode to the matching [`AppTheme`].
    fn theme_for_mode(mode: egui::Theme) -> AppTheme {
        if mode == egui::Theme::Dark {
            AppTheme::dark()
        } else {
            AppTheme::light()
        }
    }

    /// Map an [`AppTheme`] to the egui theme preference (the forward of
    /// [`Self::theme_for_mode`]).
    fn preference_for(theme: &AppTheme) -> egui::ThemePreference {
        if theme.is_dark() {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        }
    }

    /// Map an [`AppTheme`] to the egui theme mode.
    fn mode_for(theme: &AppTheme) -> egui::Theme {
        if theme.is_dark() {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        }
    }

    /// Style armas + egui for `theme`. Does NOT touch the egui theme
    /// preference — callers decide whether to pin an explicit mode
    /// (`ctx.set_theme`) or keep following System.
    fn apply_theme(theme: &AppTheme, ctx: &egui::Context) {
        ctx.set_armas_theme(theme.theme().clone());
        Self::apply_egui_style(ctx, theme);
    }

    /// Tint the egui styles with the armas theme background. Called via
    /// [`Self::apply_theme`] whenever the effective theme changes: `new`, a
    /// manual toggle, or the system-follow branch in `ui`. Both modes are
    /// tinted in place (`all_styles_mut`) so a runtime mode switch (eframe
    /// applies the wasm system theme after `new`, the prefers-color-scheme
    /// listener and native `ThemeChanged` fire later) can never activate an
    /// untinted style.
    fn apply_egui_style(ctx: &egui::Context, theme: &AppTheme) {
        let bg = theme.theme().background();
        ctx.all_styles_mut(|style| {
            style.visuals.panel_fill = bg;
            style.visuals.window_fill = bg;
            style.visuals.extreme_bg_color = bg;
        });
    }

    fn open_url(url: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url(url);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = open::that(url);
        }
    }
}

// =============================================================================
// Main Loop
// =============================================================================

impl eframe::App for ShowcaseApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Sync page from URL if it changed (handles back/forward)
        #[cfg(target_arch = "wasm32")]
        {
            let url_page = Self::page_from_url();
            if url_page != self.page_state {
                self.page_state = url_page;
            }
        }

        // Follow the system theme until the user pins one — on wasm the
        // browser theme only reaches egui after `new`, and the OS theme can
        // change later. No ctx.set_theme here: the preference must stay
        // System so egui keeps tracking the OS/browser theme; we only
        // re-style ours to match.
        let mode = ui.ctx().theme();
        if self.theme_source == ThemeSource::System && self.last_mode != Some(mode) {
            self.last_mode = Some(mode);
            self.theme = Self::theme_for_mode(mode);
            let ctx = ui.ctx();
            Self::apply_theme(&self.theme, ctx);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            // Header
            let header_response =
                SiteHeader::new(self.theme.theme(), self.theme_source.toggle_icon()).show(ui);
            self.handle_header_response(ui.ctx(), header_response);

            // Main content — mobile sidebar overlay takes priority on any page
            let is_mobile = ui.available_width() < layout::MOBILE_BREAKPOINT;
            if is_mobile && self.sidebar_open {
                self.render_mobile_sidebar(ui);
            } else {
                match self.page_state {
                    PageState::Hero => self.render_hero(ui),
                    PageState::ComponentsList => self.render_components_list(ui),
                    PageState::DocsPage(_) => self.render_docs(ui),
                }
            }
        });

        // Back/forward navigation doesn't produce egui input, so the wasm
        // shell still needs frames to re-sync the page from the URL — ask
        // for one every 100ms. Native needs nothing: egui repaints on input
        // and every animated component requests repaints itself.
        #[cfg(target_arch = "wasm32")]
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }

    /// Persist the theme source (a file on native, localStorage on wasm —
    /// eframe calls this on autosave and on exit).
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(THEME_KEY, self.theme_source.as_str().to_owned());
    }

    /// Only the theme source is persisted. egui's own memory would both
    /// fight the app's theme authority (a restored theme preference) and
    /// share the per-origin "egui_memory_ron" localStorage key with every
    /// other eframe app on the same origin.
    fn persist_egui_memory(&self) -> bool {
        false
    }
}

// =============================================================================
// Event Handlers
// =============================================================================

impl ShowcaseApp {
    /// Navigate to a page state and sync URL
    fn navigate_to(&mut self, state: PageState) {
        self.page_state = state;
        self.sync_url_to_page();
    }

    /// Sync URL hash to current page state
    fn sync_url_to_page(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            match &self.page_state {
                PageState::Hero => url_utils::url::set_hash(""),
                PageState::ComponentsList => url_utils::url::set_hash("components"),
                PageState::DocsPage(idx) => {
                    if let Some((section, component)) = showcase_gen::get_route_by_index(*idx) {
                        let route = format!("{}/{}", section, component);
                        url_utils::url::set_hash(&route);
                    }
                }
            }
        }
    }

    fn handle_header_response(
        &mut self,
        ctx: &egui::Context,
        response: components::SiteHeaderResponse,
    ) {
        if response.logo_clicked {
            self.navigate_to(PageState::Hero);
        }
        if response.docs_clicked {
            self.navigate_to(PageState::DocsPage(0));
        }
        if response.components_clicked {
            self.navigate_to(PageState::ComponentsList);
        }
        if response.github_clicked {
            Self::open_url("https://github.com/PoHsuanLai/Armas");
        }
        if response.crates_io_clicked {
            Self::open_url("https://crates.io/crates/armas");
        }
        if response.docs_rs_clicked {
            Self::open_url("https://docs.rs/armas");
        }
        if response.theme_toggle_clicked {
            self.cycle_theme(ctx);
        }
        if response.hamburger_clicked {
            self.sidebar_open = !self.sidebar_open;
        }
    }

    fn handle_sidebar_response(&mut self, response: components::SiteSidebarResponse) {
        if let Some(idx) = response.selected_page {
            self.navigate_to(PageState::DocsPage(idx));
        }
    }
}

// =============================================================================
// Render Methods
// =============================================================================

impl ShowcaseApp {
    fn render_hero(&mut self, ui: &mut egui::Ui) {
        let response = SiteHero::new(self.theme.theme()).show(ui);
        if response.get_started_clicked {
            self.navigate_to(PageState::DocsPage(0));
        }
        if response.components_clicked {
            self.navigate_to(PageState::ComponentsList);
        }
    }

    fn render_components_list(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme.theme().clone();
        let response = ComponentsListPage::new(&theme, &self.pages).show(ui);
        if let Some(idx) = response.selected_page {
            self.navigate_to(PageState::DocsPage(idx));
        }
    }

    fn render_docs(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let is_mobile = rect.width() < layout::MOBILE_BREAKPOINT;

        if is_mobile {
            // Mobile sidebar overlay is handled at top level; just show content
            self.render_content(ui);
        } else {
            self.render_desktop_docs(ui, rect);
        }
    }

    fn render_mobile_sidebar(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let theme = self.theme.theme().clone();
        ui.painter().rect_filled(rect, 0.0, theme.background());

        // Reserve bottom area for external links
        let links_height = 48.0;
        let sidebar_rect =
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - links_height));
        let links_rect =
            egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - links_height), rect.max);

        // Sidebar navigation
        let response = ui
            .scope_builder(egui::UiBuilder::new().max_rect(sidebar_rect), |ui| {
                SiteSidebar::new(&theme, &mut self.search_text, &self.pages).show(ui)
            })
            .inner;

        if response.selected_page.is_some() {
            self.sidebar_open = false;
            self.handle_sidebar_response(response);
        }

        // External links at bottom
        ui.scope_builder(egui::UiBuilder::new().max_rect(links_rect), |ui| {
            // Top border
            let line_rect =
                egui::Rect::from_min_size(links_rect.min, egui::vec2(links_rect.width(), 1.0));
            ui.painter().rect_filled(line_rect, 0.0, theme.border());

            ui.add_space(1.0);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 0))
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        if Button::new("GitHub")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Small)
                            .show(ui)
                            .clicked()
                        {
                            Self::open_url("https://github.com/PoHsuanLai/Armas");
                        }
                        if Button::new("Crates.io")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Small)
                            .show(ui)
                            .clicked()
                        {
                            Self::open_url("https://crates.io/crates/armas");
                        }
                        if Button::new("Docs.rs")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Small)
                            .show(ui)
                            .clicked()
                        {
                            Self::open_url("https://docs.rs/armas");
                        }
                    });
                });
        });
    }

    fn render_desktop_docs(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let border_color = self.theme.theme().border();
        let mut sidebar_response = None;

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_height(rect.height());

                // Sidebar
                let sr = ui.vertical(|ui| {
                    ui.set_width(layout::SIDEBAR_WIDTH);
                    ui.set_height(rect.height());
                    SiteSidebar::new(self.theme.theme(), &mut self.search_text, &self.pages)
                        .show(ui)
                });
                sidebar_response = Some(sr.inner);

                // Border line
                let line_rect = ui.allocate_space(egui::vec2(1.0, rect.height())).1;
                ui.painter().rect_filled(line_rect, 0.0, border_color);

                // Content takes remaining width
                let remaining = ui.available_width();
                ui.vertical(|ui| {
                    ui.set_width(remaining);
                    ui.set_height(rect.height());
                    self.render_content(ui);
                });
            });
        });

        if let Some(response) = sidebar_response {
            self.handle_sidebar_response(response);
        }
    }

    fn render_content(&mut self, ui: &mut egui::Ui) {
        let theme = self.theme.theme();
        let full_width = ui.available_width();

        egui::Frame::new().fill(theme.background()).show(ui, |ui| {
            ui.set_width(full_width);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(full_width);

                let content_width = full_width.min(layout::CONTENT_MAX_WIDTH);
                let h_margin = ((full_width - content_width) / 2.0).max(24.0);

                ui.horizontal(|ui| {
                    ui.add_space(h_margin);
                    ui.vertical(|ui| {
                        ui.set_max_width(content_width);
                        ui.add_space(32.0);

                        if let PageState::DocsPage(idx) = self.page_state {
                            if let Some((_, show_fn)) = self.pages.get(idx) {
                                show_fn(ui);
                            }
                        }

                        ui.add_space(64.0);
                    });
                });
            });
        });
    }
}

// =============================================================================
// Entry Point
// =============================================================================

fn main() -> eframe::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        wasm_bindgen_futures::spawn_local(async {
            use wasm_bindgen::JsCast;
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");
            let canvas = document
                .get_element_by_id("armas_canvas")
                .expect("Failed to find armas_canvas")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("armas_canvas is not a canvas");

            eframe::WebRunner::new()
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|cc| Ok(Box::new(ShowcaseApp::new(cc)))),
                )
                .await
                .expect("Failed to start eframe");
        });
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eframe::run_native(
            "Armas",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1200.0, 800.0])
                    .with_title("Armas"),
                ..Default::default()
            },
            Box::new(|cc| Ok(Box::new(ShowcaseApp::new(cc)))),
        )
    }
}
