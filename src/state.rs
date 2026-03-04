// etat du joueur
// j'utilise i32 pour hp et pas u32 parce que hp_delta peut etre negatif
// avec u32 on aurait un underflow quand le delta depasse les PV restants

pub struct GameState {
    pub current_scene: String,
    pub hp: i32,
    pub inventory: Vec<String>,
}

impl GameState {
    pub fn new(start_scene: &str, initial_hp: i32) -> Self {
        // println!("[debug] new game: scene={} hp={}", start_scene, initial_hp);
        GameState {
            current_scene: start_scene.to_string(),
            hp: initial_hp,
            inventory: Vec::new(),
        }
    }

    pub fn has_item(&self, item: &str) -> bool {
        self.inventory.contains(&item.to_string())
    }

    pub fn pick_up(&mut self, item: &str) {
        // eviter les doublons
        if !self.has_item(item) {
            self.inventory.push(item.to_string());
        }
    }

    // pas utilise pour l'instant, mais pourrait servir si on etend le
    // systeme d'inventaire (deposer un objet, le perdre apres un evenement...)
    #[allow(dead_code)]
    pub fn drop_item(&mut self, item: &str) {
        self.inventory.retain(|i| i != item);
    }
}

// idee: sauvegarder l'etat dans un fichier texte pour pouvoir reprendre une partie
// pas fini : serde_json n'est pas dans les dependances du projet
//
// fn save_to_disk(&self) -> std::io::Result<()> {
//     // non fonctionnel pour l'instant
//     let line = format!("{},{},{}", self.current_scene, self.hp,
//                        self.inventory.join(";"));
//     std::fs::write("savegame.txt", line)
// }
