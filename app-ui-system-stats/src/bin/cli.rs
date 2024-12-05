use app_ui_system_stats::SysStats;

fn main() -> std::io::Result<()> {
    let mut app = SysStats::new();
    app.refresh_all();
    println!("{:#?}",  &app );
    Ok(())
}