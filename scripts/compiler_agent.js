const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const GEMINI_API_KEY = process.env.GEMINI_API_KEY;
const MAX_ITERATIONS = 5;

async function callGemini(promptText) {
    if (!GEMINI_API_KEY) {
        throw new Error("GEMINI_API_KEY is not set in the environment.");
    }

    const url = `https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${GEMINI_API_KEY}`;

    const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            contents: [{ parts: [{ text: promptText }] }]
        })
    });

    if (!response.ok) {
        throw new Error(`Gemini API Error: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return data.candidates[0].content.parts[0].text;
}

function extractRustCode(llmResponse) {
    const match = llmResponse.match(/```rust\n([\s\S]*?)\n```/);
    if (match && match[1]) {
        return match[1];
    }
    return llmResponse; // Fallback if no block exists
}

async function main() {
    const astFile = process.argv[2];
    if (!astFile) {
        console.error("Usage: node scripts/compiler_agent.js <path_to_ast.json>");
        process.exit(1);
    }

    const astJson = fs.readFileSync(astFile, 'utf-8');
    const systemPromptTemplate = fs.readFileSync(path.join(__dirname, '../prompts/transpiler_agent.txt'), 'utf-8');

    let currentPrompt = `${systemPromptTemplate}

CRITICAL ARCHITECTURAL CHANGE:
The project now uses a wgpu-based rendering shell in \`src/main.rs\`.
Your task is NOT to overwrite \`src/main.rs\`.
Instead, you must output a Rust function named \`generate_ui_tree\` that takes a \`&mut LayoutEngine\` and returns a \`NodeId\`.
This function will be used by \`src/main.rs\` to build the Taffy layout tree.

Here is the input AST to compile:
\`\`\`json
${astJson}
\`\`\`

Please output ONLY the implementation of \`generate_ui_tree\` and any necessary helper imports for \`src/ui_gen.rs\`.
Wrap the code in a \`\`\`rust block.`;

    let iteration = 0;
    let success = false;

    while (iteration < MAX_ITERATIONS && !success) {
        iteration++;
        console.log(`\n--- Compiler Iteration ${iteration} ---`);
        console.log("Calling LLM to generate/fix code...");

        try {
            const llmResponse = await callGemini(currentPrompt);
            const rustCode = extractRustCode(llmResponse);

            // Output to a separate module to avoid overwriting main shell
            const targetFile = path.join(__dirname, '../src/ui_gen.rs');

            const fileContent = `use crate::layout::{Node, AstRect, FlexStyles, LayoutEngine};
use taffy::prelude::NodeId;
use std::collections::HashMap;

pub fn generate_ui_tree(engine: &mut LayoutEngine) -> NodeId {
${rustCode}
}`;

            fs.writeFileSync(targetFile, fileContent);
            console.log(`Wrote generated code to ${targetFile}. Running cargo check...`);

            // Compile/check
            try {
                execSync('cargo check', { stdio: 'pipe' });
                console.log("Compilation Successful!");
                success = true;
            } catch (error) {
                const compilerError = error.stderr ? error.stderr.toString() : error.message;
                console.error("Compilation Failed.");

                // Format remediation prompt
                currentPrompt = `The previous Rust code you provided for \`src/ui_gen.rs\` failed to compile. Here is the compiler output:

\`\`\`
${compilerError}
\`\`\`

Please fix the errors and provide the corrected Rust code (the body of the function or the whole function) wrapped in \`\`\`rust block.`;
            }

        } catch (error) {
            console.error("Agent execution failed:", error);
            process.exit(1);
        }
    }

    if (!success) {
        console.error(`\nFailed to compile the AST into valid Rust code after ${MAX_ITERATIONS} iterations.`);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}