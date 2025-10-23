use anyhow::Result;
use domain_model_note_taking_lib::llm_integration::LlmIntegration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the LLM integration
    // Make sure you have a .env file with LLM_PROVIDER configured
    let integration = LlmIntegration::new()?;

    // Example user request
    let user_request = r#"
Je veux modéliser un système de bibliothèque.
- Un Livre a un titre, un ISBN (unique), une date de publication et un nombre de pages
- Un Auteur a un nom et une date de naissance
- Un Livre peut avoir plusieurs Auteurs (1..n)
- Un Auteur peut écrire plusieurs Livres (0..n)
- Un Emprunt lie un Livre et un Utilisateur avec une date de début et de fin
- Invariant: La date de fin doit être après la date de début
"#;

    println!("Requesting DomainModel from LLM...\n");
    
    // Process the request - LLM will return ONLY valid DomainModel JSON
    match integration.process_request(user_request).await {
        Ok(domain_model) => {
            println!("✓ DomainModel généré avec succès!\n");
            println!("{}", serde_json::to_string_pretty(&domain_model)?);
        }
        Err(e) => {
            eprintln!("✗ Erreur: {}", e);
        }
    }

    Ok(())
}
