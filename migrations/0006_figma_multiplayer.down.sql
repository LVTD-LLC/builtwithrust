-- Change only the original catalog values; preserve independent editorial updates.
UPDATE projects
SET description = 'Figma rewrote its multiplayer sync server in Rust in 2018 to cut tail latency and memory usage, and later used Rust for parts of its rendering pipeline compiled to WebAssembly.', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'figma' AND description = 'When two people edit the same Figma file, their changes need to stay in sync. In 2018 Figma described rewriting its TypeScript multiplayer service in Rust: a Rust process handled each document, while Node.js handled network traffic. The browser canvas was a separate system: Figma described building it in C++ and compiling it to WebAssembly.';

UPDATE projects
SET stack = '["wasm"]', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
WHERE slug = 'figma' AND stack = '["rust"]';
