use stargem_backend::scenarios::pick_scenario_interactive;

fn main() -> std::io::Result<()> {
    let name = pick_scenario_interactive()?;
    println!("{}", name);
    Ok(())
}
