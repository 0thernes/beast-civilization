# frozen_string_literal: true

# `beast_yarv.rb` v0.2.0: CRuby 3.4 + Prism AST + 10-Generation YARV Live Bytecode Evolutionary Breeder (Tier T1a)
# Bridges directly into Zig `bin/beast_metal.dll` via Fiddle C-ABI with zero external gems.

require "digest"
require "fiddle"
require "prism"

module BeastYarv
  FORBIDDEN_CALLS = %i[
    system exec spawn fork open popen syscall eval instance_eval class_eval module_eval
    require_relative load autoload const_get remove_const delete unlink rmdir
  ].freeze

  # Ring-0 Prism AST Gate: walks parsed Ruby 3.4 Prism AST and blocks forbidden calls/backticks
  # BEFORE YARV bytecode compilation is permitted.
  def self.verify_prism_ast!(source)
    res = Prism.parse(source)
    raise "Prism syntax error: #{res.errors.map(&:message).join('; ')}" unless res.success?

    violations = []
    walk = lambda do |node|
      return unless node

      if node.is_a?(Prism::CallNode)
        violations << "forbidden call :#{node.name}" if FORBIDDEN_CALLS.include?(node.name)
      elsif node.is_a?(Prism::XStringNode) || node.is_a?(Prism::InterpolatedXStringNode)
        violations << "forbidden backtick shell literal"
      end
      node.compact_child_nodes.each { |child| walk.call(child) }
    end
    walk.call(res.value)
    raise "Ring-0 Prism AST blocked: #{violations.join(', ')}" unless violations.empty?

    true
  end

  # Compile a strategy to YARV binary bytecode, verify binary reload round-trip, and evaluate.
  def self.compile_and_run_yarv(source)
    verify_prism_ast!(source)
    iseq = RubyVM::InstructionSequence.compile(source, "colony_t1a_strategy.rb", "colony_t1a", 1)
    bin = iseq.to_binary
    sha = Digest::SHA256.hexdigest(bin)
    disasm_lines = iseq.disasm.lines.length
    reloaded = RubyVM::InstructionSequence.load_from_binary(bin)
    [reloaded.eval, sha, bin.bytesize, disasm_lines]
  end

  class ZigBridge
    def initialize(dll_path)
      @handle = Fiddle.dlopen(dll_path)
      @abi_version = Fiddle::Function.new(@handle["beast_metal_abi_version"], [], Fiddle::TYPE_UINT)
      @philox_u64 = Fiddle::Function.new(
        @handle["beast_philox_u64"],
        [Fiddle::TYPE_LONG_LONG, Fiddle::TYPE_LONG_LONG, Fiddle::TYPE_UINT],
        Fiddle::TYPE_LONG_LONG
      )
      @pack_5trits = Fiddle::Function.new(@handle["beast_pack_5trits"], [Fiddle::TYPE_VOIDP], Fiddle::TYPE_CHAR)
      @dot_b158_simd = Fiddle::Function.new(
        @handle["beast_dot_b158_simd"],
        [Fiddle::TYPE_VOIDP, Fiddle::TYPE_VOIDP, Fiddle::TYPE_SIZE_T],
        Fiddle::TYPE_LONG_LONG
      )
      @self_test = Fiddle::Function.new(@handle["beast_metal_self_test_all"], [Fiddle::TYPE_UINT], Fiddle::TYPE_LONG_LONG)
    end

    def abi_version = @abi_version.call
    def philox_u64(seed, ctr, dom) = @philox_u64.call(seed, ctr, dom) & 0xFFFF_FFFF_FFFF_FFFF
    def pack_5trits(arr5) = @pack_5trits.call(Fiddle::Pointer[arr5.pack("c5")]) & 0xFF

    def dot_b158_simd(packed_bytes, x_i32_arr)
      @dot_b158_simd.call(Fiddle::Pointer[packed_bytes.pack("C*")], Fiddle::Pointer[x_i32_arr.pack("l<*")], x_i32_arr.length)
    end

    def self_test_clean? = (@self_test.call(0) & 0xFFFF_FFFF_FFFF_FFFF) == 0xBE457001
  end

  # Evaluate a compiled YARV Many-Tems routing policy over 8 colony tasks (higher = better allocation).
  def self.score_policy(fn)
    tasks = [
      [0.92, 0.88, 0.20, 0.04, 1.50, 1.0],
      [0.40, 0.30, 0.80, 0.45, 0.50, -1.0],
      [0.85, 0.95, 0.30, 0.05, 1.80, 1.0],
      [0.55, 0.50, 0.65, 0.30, 0.70, -1.0],
      [0.90, 0.75, 0.15, 0.02, 1.40, 1.0],
      [0.35, 0.25, 0.85, 0.50, 0.40, -1.0],
      [0.88, 0.82, 0.25, 0.06, 1.60, 1.0],
      [0.45, 0.40, 0.70, 0.35, 0.60, -1.0]
    ]
    tasks.sum { |a, u, d, f, r, label| fn.call(a, u, d, f, r) * label }
  end

  def self.strategy_source(alpha, beta, gamma, delta, boost)
    <<~RUBY
      lambda do |a, u, d, f, r|
        s = (a ** #{alpha}) * (u ** #{beta}) * ((1.0 - d) ** #{gamma}) * ((1.0 - f) ** #{delta}) * r
        ((1.0 - f) > 0.85 && (1.0 - d) > 0.60) ? (s * #{boost}) : (s * 0.85)
      end
    RUBY
  end

  def self.run_selftest(repo_root)
    yjit_on = defined?(RubyVM::YJIT) && RubyVM::YJIT.enabled? ? true : false
    puts "=== CRUBY #{RUBY_VERSION} + PRISM + 10-GEN YARV BREEDER (YJIT=#{yjit_on}) ==="

    # 1. 10-Generation Strictly-Monotonic YARV Bytecode Evolution with Automatic Rollback!
    params = [1.0, 1.0, 0.6, 0.8, 1.00]
    best_fn, best_sha, best_bytes, best_ops = compile_and_run_yarv(strategy_source(*params))
    init_fit = score_policy(best_fn)
    best_fit = init_fit
    promotions = 0
    rollbacks = 0

    10.times do |gen|
      cand = params.dup
      # Deterministic mutation pattern: alternating improvement + deliberate bad regression to test rollback!
      if gen.even?
        cand[2] = (cand[2] + 0.18).round(3)
        cand[3] = (cand[3] + 0.22).round(3)
        cand[4] = (cand[4] + 0.10).round(3)
      else
        cand[2] = 0.05
        cand[4] = 0.50
      end
      fn, sha, bytes, ops = compile_and_run_yarv(strategy_source(*cand))
      fit = score_policy(fn)
      if fit > best_fit
        params = cand
        best_fit = fit
        best_sha = sha
        best_bytes = bytes
        best_ops = ops
        promotions += 1
      else
        rollbacks += 1
      end
    end

    raise "YARV 10-gen breeder failed to improve!" unless best_fit > init_fit && promotions >= 4 && rollbacks >= 4

    # 2. Planted Injection Mutants -> Prism Ring-0 AST Gate MUST block 2/2!
    m1 = begin; compile_and_run_yarv('system("echo pwned")'); false; rescue RuntimeError => e; e.message.include?("Ring-0 Prism AST blocked"); end
    m2 = begin; compile_and_run_yarv('`whoami`'); false; rescue RuntimeError => e; e.message.include?("Ring-0 Prism AST blocked"); end
    raise "Planted Prism AST mutant escaped!" unless m1 && m2

    puts format(
      "[PASS] YARV 10-Gen Breeder (fit %.4f -> %.4f, promotions=%d, rollbacks=%d, final=%dB/%dops sha=%s)",
      init_fit, best_fit, promotions, rollbacks, best_bytes, best_ops, best_sha[0, 12]
    )
    puts "[PASS] Prism Ring-0 AST Guard (2/2 planted injection mutants blocked before YARV compile)"

    # 3. Direct CRuby -> Zig Fiddle C-ABI bridge & latency test (KC-2 < 10,000 ns)
    dll_name = Gem.win_platform? ? "beast_metal.dll" : "libbeast_metal.so"
    dll_path = File.join(repo_root, "bin", dll_name)
    if File.exist?(dll_path)
      zig = ZigBridge.new(dll_path)
      raise "Zig ABI mismatch" unless zig.abi_version == 0x00010000
      raise "Zig self-test failed from Ruby" unless zig.self_test_clean?

      packed = zig.pack_5trits([1, -1, 0, 1, 1])
      dot = zig.dot_b158_simd([packed], [10, 20, 30, 40, 50])
      raise "CRuby->Zig b1.58 dot mismatch: #{dot}" unless dot == 80

      iters = 50_000
      t0 = Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)
      acc = 0
      i = 0
      while i < iters
        acc ^= zig.philox_u64(5000, i, 77)
        i += 1
      end
      ns_per_call = (Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond) - t0).to_f / iters
      raise "KC-2 violation: #{ns_per_call} ns" if ns_per_call > 10_000.0

      puts format("[PASS] CRuby->Zig Fiddle C-ABI (ABI=0x%08x, b1.58_dot=%d, FFI=%.1f ns/call, acc=0x%x)", zig.abi_version, dot, ns_per_call, acc & 0xFFFF)
    end
  end
end

BeastYarv.run_selftest(File.expand_path("..", __dir__)) if __FILE__ == $PROGRAM_NAME
