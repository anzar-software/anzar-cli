use console::Style;
use dialoguer::theme::ColorfulTheme;

pub fn theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: console::style("?".to_string()).cyan().bold(),
        prompt_suffix: console::style("›".to_string()).cyan().dim(),
        success_prefix: console::style("✓".to_string()).green().bold(),
        success_suffix: console::style("·".to_string()).green().dim(),
        error_prefix: console::style("✗".to_string()).red().bold(),
        active_item_prefix: console::style("›".to_string()).cyan().bold(),
        inactive_item_prefix: console::style(" ".to_string()),
        checked_item_prefix: console::style("◉".to_string()).cyan().bold(),
        active_item_style: Style::new().cyan().bold(),
        inactive_item_style: Style::new().dim(),
        prompt_style: Style::new().bold(),
        defaults_style: Style::new().dim(),
        values_style: Style::new().cyan(),
        hint_style: Style::new().dim(),
        error_style: Style::new().red(),
        ..ColorfulTheme::default()
    }
}
