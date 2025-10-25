use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::llm_router::LlmRouter;

/// User's answer to an interview question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnswer {
    pub section_id: u32,
    pub question_index: u32,
    pub question: String,
    pub answer: String,
}

/// A section of answers from the interview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewSection {
    pub section_id: u32,
    pub section_title: String,
    pub answers: Vec<UserAnswer>,
}

/// Result of processing interview answers for a section
#[derive(Debug, Serialize, Deserialize)]
pub struct SectionCanvasResult {
    pub section_id: u32,
    pub section_title: String,
    pub canvas_content: String, // Markdown content for this section
}

/// Complete canvas content
#[derive(Debug, Serialize, Deserialize)]
pub struct FullCanvasResult {
    pub markdown: String,
}

/// Interview processor that uses LLM to transform answers into canvas content
pub struct InterviewProcessor {
    llm_router: LlmRouter,
}

impl InterviewProcessor {
    pub fn new() -> Result<Self> {
        let llm_router = LlmRouter::new()?;
        Ok(Self { llm_router })
    }

    /// Process answers for a specific section and generate canvas content
    pub async fn process_section(&self, section: InterviewSection) -> Result<SectionCanvasResult> {
        let system_prompt = self.get_system_prompt_for_section(&section.section_title);
        
        // Format the Q&A into a structured prompt
        let mut qa_text = format!("Section: {}\n\n", section.section_title);
        for answer in &section.answers {
            qa_text.push_str(&format!("Q: {}\nR: {}\n\n", answer.question, answer.answer));
        }

        // Ask LLM to transform answers into canvas markdown format
        let canvas_content = self
            .llm_router
            .generate_text(&system_prompt, &qa_text)
            .await
            .context("Failed to generate canvas content from answers")?;

        Ok(SectionCanvasResult {
            section_id: section.section_id,
            section_title: section.section_title,
            canvas_content,
        })
    }

    /// Generate the complete canvas from all processed sections
    pub async fn generate_full_canvas(&self, sections: Vec<SectionCanvasResult>) -> Result<FullCanvasResult> {
        // Build the full canvas markdown
        let mut markdown = String::from("# Canvas — Rich Domain Model (DDD)\n\n");
        markdown.push_str("> Objectif : cadrer un domaine avec un modèle riche (entités porteuses de logique, invariants explicites, langage ubiquiste). Remplis court et concret.\n\n");
        markdown.push_str("---\n\n");

        // Add each section's content
        for section in sections {
            markdown.push_str(&format!("## {}\n\n", section.section_title));
            markdown.push_str(&section.canvas_content);
            markdown.push_str("\n\n");
        }

        Ok(FullCanvasResult { markdown })
    }

    /// Get section-specific system prompt to guide LLM output
    fn get_system_prompt_for_section(&self, section_title: &str) -> String {
        let base_prompt = r#"Tu es un expert Domain-Driven Design qui transforme des réponses d'interview en documentation structurée pour un Canvas Domain Model.

RÈGLES IMPORTANTES:
1. Reformule les réponses en langage métier clair
2. Sois concis et orienté résultat
3. Extrait les invariants et règles métier
4. Utilise le format markdown approprié pour la section
5. Bannir le jargon prématuré
6. Focus sur la spécificité et les exemples concrets

"#;

        let section_specific = match section_title {
            "Contexte & Vision" => r#"
FORMAT ATTENDU (markdown):
* **Problème à résoudre :** [Description concrète]
* **Valeur métier attendue :** [Impact mesurable]
* **Impacts mesurables (KPIs) :** [Métriques spécifiques]
* **Périmètre (Bounded Context) :** [Frontières claires]
"#,
            "Acteurs & Use Cases" => r#"
FORMAT ATTENDU (markdown):
* **Acteurs :** [Liste des profils concrets]
* **Top 5 use cases (orientés résultat) :**
  1. En tant que [acteur], je veux [action] afin de [bénéfice]
  2. [...]
"#,
            "Langage Ubiquiste" => r#"
FORMAT ATTENDU (markdown table):
| Terme | Définition métier | Exemple |
| ----- | ----------------- | ------- |
| [terme] | [définition] | [exemple concret] |
"#,
            "Agrégats & Entités/Value Objects" => r#"
FORMAT ATTENDU (markdown):
### Agrégats

| Agrégat | Racine | Principales entités internes | Invariants (ACID dans l'agrégat) | Politiques (domain rules) |
| ------- | ------ | --------------------------- | -------------------------------- | ------------------------- |
| [nom] | [racine] | [entités] | [invariants] | [politiques] |

### Entités & Value Objects
* **Entités (identité stable) :** [liste]
* **Value Objects (immutables, égalité structurelle) :** [liste]
"#,
            "Domain Events & Règles" => r#"
FORMAT ATTENDU (markdown table):
| Événement | Quand | Payload minimal | Consommateurs | Outbox ? |
| --------- | ----- | --------------- | ------------- | -------- |
| [EventName] | [déclencheur] | [données] | [qui réagit] | Oui/Non |

### Règles métier
[Liste des invariants et politiques]
"#,
            "Contextes & Intégration (Hexagonal)" => r#"
FORMAT ATTENDU (markdown):
* **Contextes :** [liste des bounded contexts]
* **Relations :** [type de relations entre contextes]
* **Ports sortants (domain → infra) :** [interfaces]
* **Adaptateurs :** [implémentations]
"#,
            "Sécurité, Performance, Persistance" => r#"
FORMAT ATTENDU (markdown):
* **Sécurité & Conformité:**
  - [Données sensibles et accès]
  - [Traçabilité]
* **Performance & Scalabilité:**
  - [Contraintes de concurrence]
  - [Granularité des agrégats]
* **Persistance:**
  - [Stratégie de mapping]
  - [Transactions]
"#,
            "Tests de domaine & KPI" => r#"
FORMAT ATTENDU (markdown):
### Tests métier (Given-When-Then)
```
Étant donné [état initial]
Quand [action]
Alors [résultat attendu]
Et [événement émis]
```

### KPI métier
* [Métrique 1]
* [Métrique 2]
"#,
            "Roadmap Domain-first" => r#"
FORMAT ATTENDU (markdown numbered list):
1. **MVP agrégat principal** [description]
2. **Ports/Adaptateurs critiques** [description]
3. **Projections lecture & ACL** [description]
4. **Tests scénarisés & KPI** [description]
"#,
            _ => "",
        };

        format!("{}{}", base_prompt, section_specific)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires LLM setup
    async fn test_process_section() -> Result<()> {
        let processor = InterviewProcessor::new()?;
        
        let section = InterviewSection {
            section_id: 1,
            section_title: "Contexte & Vision".to_string(),
            answers: vec![
                UserAnswer {
                    section_id: 1,
                    question_index: 0,
                    question: "Quel problème réel veux-tu résoudre ?".to_string(),
                    answer: "Gérer les commandes e-commerce avec validation des stocks".to_string(),
                }
            ],
        };

        let result = processor.process_section(section).await?;
        assert!(!result.canvas_content.is_empty());
        
        Ok(())
    }
}
