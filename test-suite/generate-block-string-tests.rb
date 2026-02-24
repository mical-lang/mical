#!/usr/bin/env ruby

# Generates block-string test cases (input.mical + output.json) based on
# dimension analysis. See block-string-test-matrix.md for details.
#
# Usage:
#   ruby test-suite/generate-block-string-tests.rb       # all phases
#   ruby test-suite/generate-block-string-tests.rb 1     # phase 1 only (0-line)
#   ruby test-suite/generate-block-string-tests.rb 2     # phase 2 only (1-line)
#   ruby test-suite/generate-block-string-tests.rb 3     # phase 3 only (multi-line)
#
# Requires: `cargo build` first (uses target/debug/mical to generate output.json)

require 'json'
require 'fileutils'
require 'open3'

PROJECT_ROOT = File.expand_path('..', __dir__)
SUITE_DIR = File.join(PROJECT_ROOT, 'test-suite')
MICAL_BIN = File.join(PROJECT_ROOT, 'target', 'debug', 'mical')

def mical_eval(input)
  stdout, stderr, status = Open3.capture3(MICAL_BIN, 'eval', '/dev/stdin', stdin_data: input)
  return [stdout, nil] if status.success?
  [nil, stderr.strip]
end

# Returns :ok, :broken, or :fail
def write_test_case(name, input, expected_output: nil)
  dir = File.join(SUITE_DIR, name)
  FileUtils.mkdir_p(dir)

  output, err = mical_eval(input)

  if output
    File.binwrite(File.join(dir, 'input.mical'), input)
    # Remove stale input.broken.mical if the case now succeeds
    broken_path = File.join(dir, 'input.broken.mical')
    FileUtils.rm_f(broken_path)

    parsed = JSON.parse(output)
    pretty = JSON.pretty_generate(parsed) + "\n"
    File.write(File.join(dir, 'output.json'), pretty)
    puts "OK     #{name}"
    return :ok
  end

  if expected_output
    File.binwrite(File.join(dir, 'input.broken.mical'), input)
    # Remove stale input.mical if present
    mical_path = File.join(dir, 'input.mical')
    FileUtils.rm_f(mical_path)

    File.write(File.join(dir, 'output.json'), expected_output)
    puts "BROKEN #{name}: #{err}"
    return :broken
  end

  File.binwrite(File.join(dir, 'input.mical'), input)
  $stderr.puts "FAIL   #{name}: #{err}"
  :fail
end

def pretty_json(hash)
  JSON.pretty_generate(hash) + "\n"
end

# --- Greedy pairwise covering array generator ---

def generate_pairwise(dimensions)
  all_pairs = {}
  dimensions.each_with_index do |d1, i|
    ((i + 1)...dimensions.size).each do |j|
      d2 = dimensions[j]
      d1.times do |v1|
        d2.times do |v2|
          all_pairs[[i, j, v1, v2]] = false
        end
      end
    end
  end

  rows = []
  until all_pairs.values.all?
    best_row = nil
    best_count = -1

    100.times do
      candidate = dimensions.map { |d| rand(d) }
      count = count_new_pairs(candidate, all_pairs, dimensions)
      if count > best_count
        best_count = count
        best_row = candidate
      end
    end

    dimensions[0].times do |v0|
      dimensions.each_with_index do |d, fi|
        next if fi == 0
        d.times do |v|
          candidate = best_row.dup
          candidate[0] = v0
          candidate[fi] = v
          count = count_new_pairs(candidate, all_pairs, dimensions)
          if count > best_count
            best_count = count
            best_row = candidate
          end
        end
      end
    end

    mark_pairs(best_row, all_pairs, dimensions)
    rows << best_row
  end
  rows
end

def count_new_pairs(row, all_pairs, dimensions)
  count = 0
  dimensions.each_with_index do |_, i|
    ((i + 1)...dimensions.size).each do |j|
      key = [i, j, row[i], row[j]]
      count += 1 if all_pairs.key?(key) && !all_pairs[key]
    end
  end
  count
end

def mark_pairs(row, all_pairs, dimensions)
  dimensions.each_with_index do |_, i|
    ((i + 1)...dimensions.size).each do |j|
      all_pairs[[i, j, row[i], row[j]]] = true
    end
  end
end

# --- Dimension values ---

STYLES = [['literal', '|'], ['folded', '>']]
CHOMPS = [['clip', ''], ['strip', '-'], ['keep', '+']]
LEADS = ['none', 'emptyln', 'wsln']
TRAILS = ['none', 'emptyln', 'wsln']
TERMS = ['dedent', 'eof', 'eof-nonl']

def indicator(style_idx, chomp_idx)
  "#{STYLES[style_idx][1]}#{CHOMPS[chomp_idx][1]}"
end

def style_name(idx); STYLES[idx][0]; end
def chomp_name(idx); CHOMPS[idx][0]; end

# --- Phase 1: 0-line block strings (42 cases, exhaustive) ---

def run_phase1
  phase1_cases = []

  STYLES.each do |(sname, schar)|
    CHOMPS.each do |(cname, cchar)|
      ind = "#{schar}#{cchar}"

      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-none-eof-nonl",
        input: "key #{ind}"
      }
      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-none-eof",
        input: "key #{ind}\n"
      }
      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-none-dedent",
        input: "key #{ind}\nother val\n",
        expected_output: pretty_json({ "key" => "", "other" => "val" })
      }
    end
  end

  STYLES.each do |(sname, schar)|
    CHOMPS.each do |(cname, cchar)|
      ind = "#{schar}#{cchar}"

      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-emptyln-eof",
        input: "key #{ind}\n\n"
      }
      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-emptyln-dedent",
        input: "key #{ind}\n\nother val\n"
      }
    end
  end

  STYLES.each do |(sname, schar)|
    CHOMPS.each do |(cname, cchar)|
      ind = "#{schar}#{cchar}"

      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-wsln-eof",
        input: "key #{ind}\n \n"
      }
      phase1_cases << {
        name: "block-string-0-#{sname}-#{cname}-wsln-dedent",
        input: "key #{ind}\n \nother val\n"
      }
    end
  end

  puts "=== Phase 1: 0-line block strings (#{phase1_cases.size} cases) ==="
  run_cases(phase1_cases)
end

# --- Phase 2: 1-line block strings (pairwise) ---

def build_1line_input(style_idx, chomp_idx, lead, trail, term)
  ind = indicator(style_idx, chomp_idx)
  lines = ["key #{ind}"]

  case lead
  when 'emptyln' then lines << ''
  when 'wsln'    then lines << ' '
  end

  lines << '  hello'

  case trail
  when 'emptyln' then lines << ''
  when 'wsln'    then lines << ' '
  end

  case term
  when 'dedent'
    lines << 'other val'
    lines.join("\n") + "\n"
  when 'eof'
    lines.join("\n") + "\n"
  when 'eof-nonl'
    lines.join("\n")
  end
end

def run_phase2
  srand(42)
  rows = generate_pairwise([2, 3, 3, 3, 3])

  supplements = [
    [0, 1, 0, 1, 1], # literal strip none emptyln eof
    [1, 2, 0, 1, 0], # folded keep none emptyln dedent
    [0, 2, 1, 2, 2], # literal keep emptyln wsln eof-nonl
    [1, 1, 1, 2, 1], # folded strip emptyln wsln eof
    [0, 0, 1, 0, 2], # literal clip emptyln none eof-nonl
  ]
  supplements.each do |s|
    rows << s unless rows.include?(s)
  end

  phase2_cases = rows.map do |row|
    si, ci, li, ti, xi = row
    name = "block-string-1-#{style_name(si)}-#{chomp_name(ci)}-#{LEADS[li]}-#{TRAILS[ti]}-#{TERMS[xi]}"
    input = build_1line_input(si, ci, li, ti, TERMS[xi])
    { name: name, input: input }
  end

  puts "=== Phase 2: 1-line block strings (#{phase2_cases.size} cases) ==="
  run_cases(phase2_cases)
end

# --- Phase 3: multi-line block strings (pairwise) ---

def build_multiline_input(style_idx, chomp_idx, lead, mid, trail, term)
  ind = indicator(style_idx, chomp_idx)
  lines = ["key #{ind}"]

  case lead
  when 'emptyln' then lines << ''
  when 'wsln'    then lines << ' '
  end

  lines << '  hello'

  case mid
  when 'emptyln' then lines << ''
  when 'wsln'    then lines << ' '
  end

  lines << '  world'

  case trail
  when 'emptyln' then lines << ''
  when 'wsln'    then lines << ' '
  end

  case term
  when 'dedent'
    lines << 'other val'
    lines.join("\n") + "\n"
  when 'eof'
    lines.join("\n") + "\n"
  when 'eof-nonl'
    lines.join("\n")
  end
end

def run_phase3
  srand(42)
  rows = generate_pairwise([2, 3, 3, 3, 3, 3])

  phase3_cases = rows.map do |row|
    si, ci, li, mi, ti, xi = row
    mids = ['none', 'emptyln', 'wsln']
    name = "block-string-multi-#{style_name(si)}-#{chomp_name(ci)}-#{LEADS[li]}-#{mids[mi]}-#{TRAILS[ti]}-#{TERMS[xi]}"
    input = build_multiline_input(si, ci, li, mids[mi], TRAILS[ti], TERMS[xi])
    { name: name, input: input }
  end

  puts "=== Phase 3: multi-line block strings (#{phase3_cases.size} cases) ==="
  run_cases(phase3_cases)
end

# --- Runner ---

$has_failures = false

def run_cases(cases)
  ok = 0
  broken = 0
  fail_count = 0
  cases.each do |c|
    result = write_test_case(c[:name], c[:input], expected_output: c[:expected_output])
    case result
    when :ok     then ok += 1
    when :broken then broken += 1
    when :fail   then fail_count += 1
    end
  end

  parts = ["#{ok} ok"]
  parts << "#{broken} broken" if broken > 0
  parts << "#{fail_count} FAIL" if fail_count > 0
  puts "Result: #{parts.join(', ')} (total #{cases.size})"

  $has_failures = true if fail_count > 0
end

phase = ARGV[0]&.to_i || 0

case phase
when 1 then run_phase1
when 2 then run_phase2
when 3 then run_phase3
else
  run_phase1
  run_phase2
  run_phase3
end

exit 1 if $has_failures
