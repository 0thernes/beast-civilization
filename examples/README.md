<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Runnable Examples (`A43`)

1. **`01_quickstart_oracle_and_c2c.py`** — Zero-dependency Python walkthrough of BitNet `b1.58` 5-trit/byte bijective packing, GGUF v3 `Q4_0` compression (`64/9x`), and the MIT `Augment-Cache` vs. `Overwrite` retention law:
   ```powershell
   python examples/01_quickstart_oracle_and_c2c.py
   ```
2. **Live OpenAI-Compatible Local GGUF/C2C Server + Endpoint-Wired Colony Mission:**
   ```powershell
   .\bin\beast.exe serve --port 11435 --gguf bin\beast_4layer_chimera.gguf
   .\bin\beast.exe agent-run --endpoint http://127.0.0.1:11435/v1/chat/completions "colony model mission"
   ```
3. **Scripted Interactive Colony REPL & MCP JSON-RPC 2.0 Stdio Bus:**
   ```powershell
   .\bin\beast.exe repl --script "probe;ad-test;mission colony model audit;quit"
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize"}' | .\bin\beast.exe mcp-serve
   ```
