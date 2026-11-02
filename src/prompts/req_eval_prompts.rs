#![allow(dead_code)]
pub const REQ_EVAL_PROMPT: &str = r#"You are a professional designer helping evaluate whether a scientific diagram satisfies a list of specific requirements.

# Input
You will be provided with:
1. A reference scientific diagram representing the target domain and layout.
2. A generated diagram that attempts to recreate or adapt the reference.
3. A list of numbered requirements.

# Task
Evaluate whether the generated diagram fully satisfies EACH requirement. For each requirement, respond with 'Yes' if satisfied or 'No' if not.

Provide your reasoning and conclusion for each requirement in JSON format as follows:
[
  {
    "requirement_id": 1,
    "reasoning": "...",
    "satisfied": true/false
  },
  ...
]
"#;
