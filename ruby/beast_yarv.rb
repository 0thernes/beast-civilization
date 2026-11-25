# frozen_string_literal: true

# `beast_yarv.rb`: CRuby 3.4 + Prism AST + YARV Live Bytecode Mutation Layer (Tier T1a)
# Bridges directly into Zig `bin/beast_metal.dll` via Fiddle C-ABI with zero external gems.

require "digest"
require "fiddle"
require "fiddle/import"
require "prism"

module BeastYarv
  FORBIDDEN_CALLS = %i[
    system exec spawn fork open popen syscall eval instance_eval class_eval module_eval
    require_relative load autoload const_get remove_const delete unlink rmdir
  ].freeze

  # Ring-0 Prism AST Gate: walks the parsed Ruby 3.4 Prism AST and rejects any forbidden call
  # or command-literal backtick before YARV bytecode compilation is even allowed.
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

  # Compile a colony strategy proc into live YARV bytecode, serialize to binary, reload from binary,
  # and return [proc_result, bytecode_sha256, bytecode_bytes, disasm_lines].
  def self.compile_and_run_yarv(source)
    verify_prism_ast!(source)
    iseq = RubyVM::InstructionSequence.compile(source, "colony_t1a_strategy.rb", "colony_t1a", 1)
    bin = iseq.to_binary
    sha = Digest::SHA256.hexdigest(bin)
    disasm_lines = iseq.disasm.lines.length

    # Reload strictly from serialized YARV binary (proves portable bytecode round-trip!)
    reloaded = RubyVM::InstructionSequence.load_from_binary(bin)
    val = reloaded.eval
    [val, sha, bin.bytesize, disasm_lines]
  end

  # Direct C-ABI Fiddle binding into Zig `beast_metal.dll`
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

    def abi_version
      @abi_version.call
    end

    def philox_u64(seed, ctr, domain)
      @philox_u64.call(seed, ctr, domain) & 0xFFFF_FFFF_FFFF_FFFF
    end

    def pack_5trits(arr5)
      buf = arr5.pack("c5")
      @pack_5trits.call(Fiddle::Pointer[buf]) & 0xFF
    end

    def dot_b158_simd(packed_bytes, x_i32_arr)
      w_str = packed_bytes.pack("C*")
      x_str = x_i32_arr.pack("l<*")
      @dot_b158_simd.call(Fiddle::Pointer[w_str], Fiddle::Pointer[x_str], x_i32_arr.length)
    end

    def self_test_clean?
      (@self_test.call(0) & 0xFFFF_FFFF_FFFF_FFFF) == 0xBE457001
    end
  end

  def self.run_selftest(repo_root)
    yjit_on = defined?(RubyVM::YJIT) && RubyVM::YJIT.enabled? ? true : false
    puts "=== CRUBY #{RUBY_VERSION} + PRISM + YARV (YJIT=#{yjit_on}) ==="

    # 1. Compile Gen-0 colony strategy to YARV binary bytecode & execute
    gen0_src = <<~RUBY
      lambda do |affinity, urgency, difficulty, failure_rate, reward|
        (affinity ** 1.0) * (urgency ** 1.2) * ((1.0 - difficulty) ** 0.8) * ((1.0 - failure_rate) ** 1.5) * reward
      end
    RUBY
    proc0, sha0, bytes0, lines0 = compile_and_run_yarv(gen0_src)
    score0 = proc0.call(0.85, 0.90, 0.25, 0.05, 1.40)

    # 2. Live T1a Bytecode Mutation -> Gen-1 adaptive margin-boosted policy
    gen1_src = <<~RUBY
      lambda do |affinity, urgency, difficulty, failure_rate, reward|
        base = (affinity ** 1.0) * (urgency ** 1.2) * ((1.0 - difficulty) ** 0.8) * ((1.0 - failure_rate) ** 1.5) * reward
        margin_boost = (1.0 - failure_rate) > 0.90 ? 1.12 : 0.95
        base * margin_boost
      end
    RUBY
    proc1, sha1, bytes1, lines1 = compile_and_run_yarv(gen1_src)
    score1 = proc1.call(0.85, 0.90, 0.25, 0.05, 1.40)

    raise "YARV mutation SHA did not change!" if sha0 == sha1
    raise "YARV mutated score invalid!" unless score1 > score0

    # 3. Planted Mutant: Malicious AST injection MUST be caught by Prism Ring-0 gate!
    mutant_caught = false
    begin
      compile_and_run_yarv('system("echo pwned")')
    rescue RuntimeError => e
      mutant_caught = e.message.include?("Ring-0 Prism AST blocked")
    end
    raise "Planted Prism AST mutant escaped!" unless mutant_caught

    backtick_caught = false
    begin
      compile_and_run_yarv('`whoami`')
    rescue RuntimeError => e
      backtick_caught = e.message.include?("Ring-0 Prism AST blocked")
    end
    raise "Planted backtick mutant escaped!" unless backtick_caught

    puts "[PASS] YARV Gen0->Gen1 live bytecode mutation (gen0=#{bytes0}B/#{lines0}ops sha=#{sha0[0, 12]} score=#{score0.round(4)} -> gen1=#{bytes1}B/#{lines1}ops sha=#{sha1[0, 12]} score=#{score1.round(4)})"
    puts "[PASS] Prism Ring-0 AST Guard (2/2 planted injection mutants blocked before YARV compile)"

    # 4. Live CRuby -> Zig Fiddle C-ABI bridge & latency measurement (Kill Criterion KC-2 < 10us)
    dll_name = Gem.win_platform? ? "beast_metal.dll" : "libbeast_metal.so"
    dll_path = File.join(repo_root, "bin", dll_name)
    if File.exist?(dll_path)
      zig = ZigBridge.new(dll_path)
      abi = zig.abi_version
      raise "Unexpected Zig ABI: 0x#{abi.to_s(16)}" unless abi == 0x00010000
      raise "Zig self-test failed from CRuby Fiddle!" unless zig.self_test_clean?

      packed = zig.pack_5trits([1, -1, 0, 1, 1])
      dot = zig.dot_b158_simd([packed], [10, 20, 30, 40, 50])
      # 1*10 + (-1)*20 + 0*30 + 1*40 + 1*50 = 80
      raise "CRuby->Zig b1.58 dot mismatch: expected 80, got #{dot}" unless dot == 80

      # Measure CRuby -> Zig FFI latency over 50,000 calls
      iters = 50_000
      t0 = Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)
      acc = 0
      i = 0
      while i < iters
        acc ^= zig.philox_u64(5000, i, 77)
        i += 1
      end
      t1 = Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)
      ns_per_call = (t1 - t0).to_f / iters
      raise "KC-2 violation: CRuby->Zig FFI took #{ns_per_call.round(1)}ns (> 10,000ns)" if ns_per_call > 10_000.0

      puts format(
        "[PASS] CRuby->Zig Fiddle C-ABI (ABI=0x%08x, b1.58_dot=%d, FFI=%.1f ns/call, acc=0x%x)",
        abi, dot, ns_per_call, acc & 0xFFFF
      )
    else
      puts "[SKIP] Zig DLL not found at #{dll_path}"
    end
  end
end

if __FILE__ == $PROGRAM_NAME
  repo_root = File.expand_path("..", __dir__)
  BeastYarv.run_selftest(repo_root)
end
