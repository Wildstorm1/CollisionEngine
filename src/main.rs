use bevy::prelude::App;
use bevy::DefaultPlugins;
use bevy::app::{ Startup, Update };

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, test_startup)
        .add_systems(Update, test_update)
        .run();
}

fn test_startup() {
    println!("Hello, startup!");
}

fn test_update() {
    println!("Hello, update!");
}
