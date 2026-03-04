// TP2 -- jeu narratif YAML
// le main fait juste la boucle de jeu, la logique est dans les modules

mod commands;
mod errors;
mod scenario;
mod state;

use std::io::{self, BufRead, Write};
use commands::{parse_command, CommandOutcome, GameCommand, LookCommand};
use errors::GameError;

fn main() {
    // charger story.yaml depuis le repertoire courant
    // si le fichier est pas la, on quitte directement
    let scenario = match scenario::Scenario::load("story.yaml") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur au chargement du scenario: {}", e);
            std::process::exit(1);
        }
    };

    let mut state = state::GameState::new(&scenario.start_scene, scenario.initial_hp);

    println!("=== Jeu Narratif ===");
    println!("commandes: look | choose <n> | inventory | status | quit");
    println!("--------------------\n");

    // afficher la scene de depart
    // j'appelle execute() directement parce que c'est un LookCommand simple
    let _ = LookCommand.execute(&scenario, &mut state);

    let stdin = io::stdin();

    // boucle principale
    // j'avais essaye avec un Iterator<Item=String> mais c'etait complique
    // avec les lifetimes, j'ai fait simple
    loop {
        print!("\n> ");
        io::stdout().flush().unwrap(); // flush sinon le prompt s'affiche pas

        let mut ligne = String::new();
        match stdin.lock().read_line(&mut ligne) {
            Ok(0) => break, // EOF (Ctrl+D)
            Ok(_) => {}
            Err(_) => break,
        }

        if ligne.trim().is_empty() {
            continue;
        }

        // parser la commande
        let cmd = match parse_command(&ligne) {
            Ok(c)  => c,
            Err(e) => {
                println!("Erreur: {}", e);
                continue;
            }
        };

        // executer la commande et traiter le resultat
        match cmd.execute(&scenario, &mut state) {
            Ok(CommandOutcome::Continue) => {}

            Ok(CommandOutcome::Quit) => {
                println!("A bientot.");
                break;
            }

            Ok(CommandOutcome::Victory) => {
                println!("\n*** VICTOIRE ! Vous avez reussi. ***");
                break;
            }

            Ok(CommandOutcome::Escape) => {
                println!("\n*** Vous vous echappez. Fin de partie. ***");
                break;
            }

            Ok(CommandOutcome::Defeat) => {
                println!("\n*** Defaite. Fin de partie. ***");
                break;
            }

            Err(GameError::GameOver) => {
                println!("\n*** GAME OVER. Vos PV sont tombes a zero. ***");
                break;
            }

            Err(GameError::InvalidChoice(n)) => {
                println!("Choix {} invalide. Tapez 'look' pour voir les options.", n);
            }

            Err(GameError::MissingItem(ref obj)) => {
                println!("Il vous faut '{}' pour faire ca.", obj);
            }
        }
    }

    // println!("[debug] fin de partie, scene finale: {}", state.current_scene);
}
