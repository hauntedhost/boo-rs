use rand::Rng;

pub fn generate_room_name() -> String {
    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(10..99);

    format!(
        "{}-{}-{}-{}",
        random_adjective(),
        random_adjective(),
        random_noun(),
        random_num
    )
}

pub fn generate_username() -> String {
    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(10..99);

    format!("{}-{}-{}", random_adjective(), random_noun(), random_num)
}

fn random_adjective() -> String {
    get_random(vec![
        "ancient", "barren", "chilly", "distant", "eerie", "frozen", "haunted", "hidden", "hollow",
        "lonely", "misty", "moody", "mystic", "quiet", "secret", "silent", "shrouded", "stark",
        "subtle", "sullen", "veiled", "velvet", "windy",
    ])
}

fn random_noun() -> String {
    get_random(vec![
        "ash", "beam", "blossom", "castle", "cliff", "cloud", "crow", "crypt", "dust", "field",
        "flame", "fog", "frost", "ghost", "gloom", "glow", "grave", "leaf", "marsh", "mist",
        "moon", "night", "path", "raven", "root", "ruin", "sage", "shade", "snow", "star", "stone",
        "storm", "stream", "thorn", "wolf",
    ])
}

fn get_random(list: Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..list.len());
    list[index].to_string()
}
