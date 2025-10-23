use anyhow::{Context, Result};
use serde_json::Value;

use crate::llm_router::LlmRouter;

/// Integration layer that uses LLM to generate DomainModel JSON
/// The LLM is constrained to only output valid DomainModel schema
pub struct LlmIntegration {
    llm_router: LlmRouter,
}

impl LlmIntegration {
    pub fn new() -> Result<Self> {
        let llm_router = LlmRouter::new()?;
        Ok(Self { llm_router })
    }

    /// Process a user request through the LLM and execute the resulting tool calls
    /// Returns the final results from executing the tools
    pub async fn process_request(&self, user_request: &str) -> Result<Value> {
        // System prompt constrains LLM to only output valid DomainModel JSON
        let system_prompt = r#"
Tu es un normalizer de Domain Model. Rends UNIQUEMENT un JSON valide DomainModel conforme au schema. Interdis les champs non listés.

Schema DomainModel (STRICT - aucun champ supplémentaire autorisé):
{
  "entities": [ /* obligatoire */
    {
      "id": "string (pattern: ^[a-zA-Z][a-zA-Z0-9_]*$)",
      "name": "string",
      "description": "string (optional)",
      "attributes": [ /* obligatoire, minItems: 1 */
        {
          "name": "string (pattern: ^[a-zA-Z][a-zA-Z0-9_]*$)",
          "type": "string|number|integer|boolean|date|datetime|email|url|uuid|json|text",
          "description": "string (optional)",
          "required": boolean (optional),
          "unique": boolean (optional),
          "defaultValue": any (optional),
          "validation": { /* optional */
            "minLength": integer,
            "maxLength": integer,
            "min": number,
            "max": number,
            "pattern": "string",
            "enum": array,
            "custom": "string"
          }
        }
      ],
      "primaryKey": ["string"] (optional),
      "uniqueConstraints": [ /* optional */
        {
          "name": "string",
          "attributes": ["string"]
        }
      ]
    }
  ],
  "relations": [ /* obligatoire */
    {
      "id": "string (pattern: ^[a-zA-Z][a-zA-Z0-9_]*$)",
      "name": "string",
      "description": "string (optional)",
      "from": {
        "entityId": "string",
        "attribute": "string (optional)",
        "label": "string (optional)"
      },
      "to": {
        "entityId": "string",
        "attribute": "string (optional)",
        "label": "string (optional)"
      },
      "cardinality": {
        "from": "0..1|1|0..n|1..n|*",
        "to": "0..1|1|0..n|1..n|*"
      },
      "required": boolean (optional),
      "cascadeDelete": boolean (optional)
    }
  ],
  "invariants": [ /* obligatoire */
    {
      "id": "string (pattern: ^[a-zA-Z][a-zA-Z0-9_]*$)",
      "name": "string",
      "description": "string (optional)",
      "type": "uniqueness|referential_integrity|domain_constraint|cardinality|business_rule|temporal|aggregation",
      "scope": { /* optional */
        "entities": ["string"],
        "relations": ["string"]
      },
      "expression": "string",
      "severity": "error|warning|info" (optional),
      "errorMessage": "string (optional)"
    }
  ]
}

RÈGLES STRICTES:
1. AUCUN champ en dehors de ce schema
2. Tous les champs "obligatoire" DOIVENT être présents
3. Les types enum DOIVENT correspondre exactement
4. Les patterns regex DOIVENT être respectés
5. Réponds UNIQUEMENT avec ce JSON, pas de tool_calls
"#;

        // Get DomainModel JSON directly from LLM
        let domain_model = self
            .llm_router
            .generate_domain_model(system_prompt, user_request)
            .await
            .context("Failed to generate DomainModel from LLM")?;

        // Convert DomainModelResponse to JSON Value
        let model_json = serde_json::to_value(&domain_model)
            .context("Failed to serialize DomainModel")?;

        Ok(model_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires environment variables (LLM_PROVIDER, etc.)
    async fn test_integration_flow() -> Result<()> {
        let integration = LlmIntegration::new()?;

        let user_request = "User entity has email and password attributes";
        let result = integration.process_request(user_request).await?;
        
        // Result should be a valid DomainModel with entities, relations, invariants
        assert!(result.get("entities").is_some());
        assert!(result.get("relations").is_some());
        assert!(result.get("invariants").is_some());
        
        Ok(())
    }
}
