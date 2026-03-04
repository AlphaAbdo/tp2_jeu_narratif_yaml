// chargement + validation du fichier story.yaml
// serde_yaml fait le gros du travail

use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub label: String,
    pub next: String,
    pub required_item: Option<String>,
}

// une scene -- certains champs sont optionnels selon le type de scene
#[derive(Debug, Deserialize, Clone)]
pub struct Scene {
    pub id: String,
    pub title: String,
    pub text: String,
    pub choices: Option<Vec<Choice>>,  // pas de choices si c'est une fin
    pub found_item: Option<String>,
    pub hp_delta: Option<i32>,
    pub ending: Option<String>, // "victory" / "escape" / "defeat"
}

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub start_scene: String,
    pub initial_hp: i32,
    pub scenes: Vec<Scene>,
}

impl Scenario {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("impossible de lire '{}': {}", path, e))?;

        let sc: Scenario = serde_yaml::from_str(&content)
            .map_err(|e| format!("erreur de parsing YAML: {}", e))?;

        // println!("[debug] {} scenes chargees", sc.scenes.len());

        sc.validate()?;
        Ok(sc)
    }

    fn validate(&self) -> Result<(), String> {
        let mut seen: HashSet<&str> = HashSet::new();

        // passe 1: verifier les IDs uniques et construire l'ensemble des IDs connus
        for s in &self.scenes {
            if !seen.insert(s.id.as_str()) {
                return Err(format!("ID duplique: '{}'", s.id));
            }
        }

        // passe 2: verifier start_scene
        // j'aurais pu faire ca dans la passe 1 mais c'est plus lisible separe
        if !seen.contains(self.start_scene.as_str()) {
            return Err(format!(
                "start_scene '{}' introuvable dans les scenes",
                self.start_scene
            ));
        }

        // passe 3: verifier que toutes les destinations choices.next existent
        for s in &self.scenes {
            if let Some(choices) = &s.choices {
                for c in choices {
                    if !seen.contains(c.next.as_str()) {
                        return Err(format!(
                            "scene '{}': destination inconnue '{}'",
                            s.id, c.next
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_scene(&self, id: &str) -> Option<&Scene> {
        // j'aurais pu utiliser une HashMap pour etre plus efficace
        // mais avec le nb de scenes qu'on a, l'iter lineaire ca change rien
        self.scenes.iter().find(|s| s.id == id)
    }

    // petite fonction pour savoir combien de scenes il y a
    // TODO: peut-etre afficher ca au demarrage du jeu ?
    #[allow(dead_code)]
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }
}

// ------ tests ------
// j'ai fait des tests pour les 5 cas demandes dans l'enonce

#[cfg(test)]
mod tests {
    use super::*;

    // helper pour pas recopier le constructeur partout
    fn scene(id: &str, next: Option<&str>, ending: Option<&str>) -> Scene {
        Scene {
            id: id.to_string(),
            title: format!("titre {}", id),
            text: "texte de test".to_string(),
            choices: next.map(|n| vec![Choice {
                label: "avancer".to_string(),
                next: n.to_string(),
                required_item: None,
            }]),
            found_item: None,
            hp_delta: None,
            ending: ending.map(str::to_string),
        }
    }

    #[test]
    fn test_validation_ok() {
        let sc = Scenario {
            start_scene: "a".to_string(),
            initial_hp: 10,
            scenes: vec![
                scene("a", Some("b"), None),
                scene("b", None, Some("victory")),
            ],
        };
        assert!(sc.validate().is_ok());
    }

    #[test]
    fn test_start_scene_inexistante() {
        let sc = Scenario {
            start_scene: "xxx".to_string(),
            initial_hp: 10,
            scenes: vec![scene("a", None, Some("victory"))],
        };
        let err = sc.validate().unwrap_err();
        assert!(err.contains("xxx"), "le message d'erreur doit mentionner 'xxx'");
    }

    #[test]
    fn test_destination_inconnue() {
        let sc = Scenario {
            start_scene: "a".to_string(),
            initial_hp: 10,
            scenes: vec![scene("a", Some("nulle_part"), None)],
        };
        assert!(sc.validate().is_err());
    }

    #[test]
    fn test_ids_en_double() {
        let sc = Scenario {
            start_scene: "a".to_string(),
            initial_hp: 10,
            scenes: vec![
                scene("a", None, Some("victory")),
                scene("a", None, Some("defeat")), // meme ID, doit planter
            ],
        };
        assert!(sc.validate().is_err());
    }
}
