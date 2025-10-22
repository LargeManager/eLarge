
# eLarge — snippet-first, code-first editor (early preview)

## Short description
A snippet-first, code-first editor written in Rust. Inspired by Helix internals, Kakoune’s selection-first model, Vim modes, Tree-sitter, cursor-tab inserts, and Bevy ECS for system design. Early preview / proof-of-concept.

## Table of contents
- Project overview
- How it works
- Actions (modes)
- Workflow examples
- Syntax transpiler (interactive)
- Advantages
- Small code examples
- Project goals
- Roadmap
- How to run / build (basic)
- Contributing
- License

## Project overview
eLarge treats everything as a snippet — structured, language-agnostic editing where actions operate on language entities (functions, classes, variables, etc.). Snippets are first-class objects; syntax is a presentation/transform layer.

## How it works
- Snippet model: each entity has predefined components (name, params, result, body, etc.).
- Actions operate on entities and their components (create, delete, swap, refactor, find, goto).
- Editor provides interactive syntax transforms so users can map preferred notation to target-language syntax.
- Language-agnostic operations via snippet component definitions and transforms.

## Actions (modes)
- Create (c)  
- Delete (d)  
- Swap (s)  
- Refactor (r)  
- Find (f)  
- Goto (g)  

## Workflow
1. Choose an action (single key).  
2. Choose an entity (two-key short code). Example: `c + f` = create function; `d + f` = delete function.  
3. Inside an entity use action + component initial. Example: `d + n` = delete function name; `s + n` = swap function name.

## Snippet model example (function)
- Components: Name, Params, Result, Body  
- Commands:  
  - Create function: `c f`  
  - Delete function name: `d n`  
  - Swap parameter order: `s p`  

## Syntax transpiler (interactive)
- Map one syntax form to another at runtime.  
- Transform declarations, rename keywords, reorder param/type positions.  
- Example: write in a preferred style; transpiler emits valid target-language code.

## Examples

### C# original
```csharp
public int Add(int num1, int num2)
{
    int sum = num1 + num2;
    return sum;
}

