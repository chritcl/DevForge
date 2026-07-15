/// 独立绑定生成入口
///
/// 运行方式：cargo run -p devforge-desktop --bin export_bindings
/// 输出：apps/desktop/src/bindings.ts（基于 CARGO_MANIFEST_DIR 构造）
fn main() -> anyhow::Result<()> {
    devforge_desktop_lib::export_bindings()?;
    println!("绑定已导出");
    Ok(())
}
