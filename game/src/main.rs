mod plugins;

fn main() {
    plugins::register_placeholders();
    println!("Feverfall app skeleton ready. Bevy 0.19 integration starts after Checkpoint 0.");
}
