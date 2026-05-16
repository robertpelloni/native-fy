# native-fy

Yes, this is an incredible use case for Google Jules. Because Jules is an **asynchronous coding agent** that clones your repo into an isolated cloud VM, plans multi-file changes, runs compilation checks, and pushes clean pull requests, it is *perfectly* suited to handle the grueling, multi-step process of building this transpiler pipeline.

Furthermore, Jules natively reads an `AGENTS.md` file in the root of your repository to understand project-specific rules, architectural boundaries, and custom APIs.

Here is the exact blueprint, repository setup, and system prompts you need to feed into Google Jules to have it build your AI Native UI "Transpiler" engine.

---

## Step 1: Prepare the Repository Structure

Before prompting Jules, initialize a clean repository with this structure so the agent knows exactly where to write code:

```text
├── AGENTS.md                  # This instructs Jules on the overall system architecture
├── scripts/
│   └── web_scraper.ts         # Script for Phase 1 (Playwright structural extraction)
├── src/
│   ├── main.rs                # Rust Core / Windowing / wgpu setup
│   ├── layout.rs              # Taffy layout integration
│   └── runtime.js             # QuickJS bridge interface
└── prompts/
    └── transpiler_agent.txt   # Prompt templates Jules will use for Phase 2

```

---

## Step 2: Create the `AGENTS.md` File

Jules will parse this file immediately upon boot. This prevents the Gemini model from hallucinating or defaulting to standard web HTML/CSS paradigms. Copy this exactly into the root of your project:

```markdown
# Project Architecture: AI Native UI Engine

This project builds an ultra-lightweight desktop runtime that bypasses Chromium by compiling layout trees into a native Rust GPU-rendered scene graph.

## Strict Technology Stack Constraints
1. **Rust Core:** `winit` for windowing, `wgpu` for hardware-accelerated rendering, and `taffy` for layout computation.
2. **JavaScript Engine:** `QuickJS` via Rust bindings to run application business logic.
3. **No Web Engines:** Absolute ban on Chromium, WebKit, or Edge WebView2. Do not use standard HTML DOM elements or CSS properties outside of the Taffy specification.

## Core Translation Abstractions
All layouts must compile strictly down to a structural JSON AST matching this shape:
- Primitives allowed: `Box`, `Text`, `Image`, `Input`, `List`
- Styling allowed: Flexbox properties only (via Taffy mapping: `flex-direction`, `padding`, `margin`, `align-items`, `justify-content`).

## Jules Operational Objective
Your job is to build the continuous self-healing loop:
1. Parse a structural JSON AST generated from a web scraping pass.
2. Use an internal code-generation loop to translate that AST into the native `wgpu` + `taffy` Rust UI tree.
3. Compile and test the output in your cloud VM. If compilation fails, analyze the compiler logs and execute a self-correcting rewrite pass.

```

---

## Step 3: Prompt Google Jules to Build the System

Now, open the Jules Web UI (`jules.google.com`) or use the Jules CLI (`jules remote new --repo <your-repo>`), select your branch, and execute this comprehensive task prompt.

### The Master Prompt for Jules

```text
Task: Implement the core multi-phase "Native-fy" transpilation pipeline as outlined in AGENTS.md.

Go through the following engineering phases step-by-step:

PHASE 1: STRUCTURAL AST EXTRACTION
- Inside `scripts/web_scraper.ts`, implement a Playwright script that takes a URL, navigates to it, and extracts the layout topology.
- It must ignore web-specific clutter and output a clean, normalized JSON Abstract Syntax Tree (AST) representing only essential structural boxes, layout coordinates, padding, margins, flexbox properties, text values, and bounding inputs.

PHASE 2: THE RUST CORE RUNTIME UTILITIES
- Inside `src/layout.rs`, implement the data structures that map the extracted structural JSON AST directly to Taffy flexbox layout nodes.
- Ensure type safety so that any layout property outside of your supported Taffy flexbox subset throws a clear, predictable validation error.

PHASE 3: THE COMPILER & SELF-HEALING ENGINE
- Create an automated agent script inside your pipeline that handles the LLM generation loop.
- This script must take the JSON AST, call the Gemini API using the template in `prompts/transpiler_agent.txt`, and output the generated layout code directly to the UI layer.
- CRITICAL: Wrap this invocation in a shell execution loop that runs `cargo check` and `cargo test` inside your current cloud VM environment. 
- If `cargo` returns a non-zero exit status, capture the stdout/stderr compiler traces, format them into a remediation prompt, feed them back to the model, and recursively patch the source files until the project builds successfully.

Deliverables:
- Fully implemented scripts/web_scraper.ts
- Fully implemented src/layout.rs with structural mapping
- An automated self-correcting shell loop for cloud VM execution
- A verified passing status from `cargo check`. Provide the plan and diffs for approval.

```

---

## Step 4: The Steering & Review Loop

Once you click **"Give me a plan"**, Jules will spin up its temporary cloud VM and give you an architectural breakdown of the changes it intends to make across your TypeScript scraper and Rust runtime engine.

1. **Review the Plan:** Ensure Jules isn't sneaking in heavy dependencies like Chromium-based wrappers. Look for pure `wgpu` and `taffy` primitives.
2. **Steer the Agent:** If Jules forgets to handle text bounding or emoji support (which we know is brutally hard), use the **User Steerability** box right in the UI to say:
> *"Update the plan for `src/layout.rs` to include font-shaping fallbacks via `cosmic-text` so the generated text doesn't crash the GPU tree."*


3. **Approve and Pull Request:** Let Jules run the compilation cycles in its isolated cloud VM. Once the tests pass and it hits zero errors, it will automatically open a polished Pull Request directly in your GitHub repository. Review the diff, merge it, and your foundational AI compiler engine is live.
