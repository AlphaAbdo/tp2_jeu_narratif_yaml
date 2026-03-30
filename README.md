# Etudiant: 
 Abdelkader Kantaoui
# TP 2 - Jeu Narratif à Embranchements (YAML + Serde)

### Sujet

Dans ce TP, vous allez implémenter un moteur de jeu narratif à embranchements. Le scénario du jeu sera défini dans un fichier YAML, et votre moteur devra charger ce scénario, le valider, et permettre au joueur d'interagir avec lui via une interface en ligne de commande.

### Format YAML attendu (exemple)
```yaml
start_scene: entrance
initial_hp: 10

scenes:
  - id: entrance
    title: Porte Principale
    text: "..."
    choices:
      - label: Entrer
        next: hall

  - id: hall
    title: Hall
    text: "..."
    choices:
      - label: Monter
        next: roof
        required_item: badge

  - id: roof
    title: Toit
    text: "..."
    ending: victory
```

### Commandes CLI attendues
- `look`: réaffiche la scène courante
- `choose <n>`: choisit l'option numéro `n`
- `inventory`: affiche l'inventaire
- `status`: affiche HP + scène
- `quit`: quitte la partie

### Travail demandé
1. Charger et parser `story.yaml` avec `serde` + `serde_yaml`.
2. Valider le scénario:
   - `start_scene` existante
   - IDs de scènes uniques
   - toutes les destinations de `choices.next` existent
3. Implémenter `parse_command(line: &str) -> Result<Box<dyn GameCommand>, ParseError>` pour transformer l'entrée utilisateur en objet commande.
4. Appliquer le Command Pattern:
   - définir un `trait` `GameCommand` avec `execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError>`
   - créer une struct par commande (`LookCommand`, `ChooseCommand`, `InventoryCommand`, `StatusCommand`, `QuitCommand`, ...)
   - implémenter `GameCommand` pour chaque struct
5. Organiser le code en modules.

### Scénarios de test minimum
1. Chemin nominal vers `Victory`.
2. Choix invalide: `choose 99` -> `Err(InvalidChoice)`.
3. Choix conditionnel sans objet -> `Err(MissingItem(_))`.
4. Perte de HP (`hp <= 0`) -> `GameOver`.
5. Scénario YAML invalide -> erreur de validation.

### Critères qualité
- `cargo check` sans erreur
- `cargo test` vert
- Messages utilisateur lisibles
- Pas de logique métier dans `main.rs`
