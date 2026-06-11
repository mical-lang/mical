#!/usr/bin/env ruby

# Generates systematic block-string test cases (input.mical + output.json)
# per the spec (doc/src/specification/block_strings.md). Expected output is
# computed in pure Ruby, independently of the Rust implementation.
#
# Generated cases are named `block-string-gen-*` and are regenerated from
# scratch on every run:
#
#   ruby test-suite/generate-block-string-tests.rb

require 'json'
require 'fileutils'

SUITE_DIR = __dir__

STYLES = { 'literal' => '|', 'folded' => '>' }.freeze
CHOMPS = { 'clip' => '', 'strip' => '-', 'keep' => '+' }.freeze

# Body patterns as content lines relative to the base indent. `:empty` is a
# completely empty line, `:wsln` a whitespace-only line (space count is
# irrelevant). Plain strings may carry extra leading spaces (more-indented
# lines).
BODIES = {
  'simple' => ['a', 'b'],
  'extra-indent' => ['a', '  b', 'c'],
  'emptyln' => ['a', :empty, 'b'],
  'wsln' => ['a', :wsln, 'b'],
  'trailing-empty' => ['a', :empty, :empty],
  'tab-content' => ["x\ty", "\tz"],
}.freeze

I_BASE = 2

# Renders the body into source lines (each without its terminator).
def render_body(content_lines)
  content_lines.map do |line|
    case line
    when :empty then ''
    when :wsln then ' ' * (I_BASE + 2)
    else (' ' * I_BASE) + line
    end
  end
end

# Content of each line as eval sees it (blank lines are empty lines).
def content_of(line)
  line.is_a?(Symbol) ? '' : line
end

def literal_join(lines_with_nl)
  lines_with_nl.map { |(content, has_nl)| content + (has_nl ? "\n" : '') }.join
end

def folded_join(lines_with_nl)
  out = +''
  prev = nil # nil | :content | :indented | :empty
  lines_with_nl.each do |(content, _has_nl)|
    if content.empty?
      out << "\n"
      prev = :empty
    else
      indented = content.start_with?(' ')
      separator =
        if prev.nil? || prev == :empty then ''
        elsif prev == :indented || indented then "\n"
        else ' '
        end
      out << separator << content
      prev = indented ? :indented : :content
    end
  end
  # The final line break of the last content line, if present in the source.
  has_content = lines_with_nl.any? { |(c, _)| !c.empty? }
  out << "\n" if has_content && lines_with_nl.last.last
  out
end

def apply_chomp(body, chomp)
  case chomp
  when 'keep' then body
  when 'strip' then body.sub(/\n+\z/, '')
  when 'clip'
    stripped = body.sub(/\n+\z/, '')
    stripped.empty? ? '' : stripped + "\n"
  else raise "unknown chomp: #{chomp}"
  end
end

def eval_block(style, chomp, content_lines, last_has_nl)
  lines_with_nl = content_lines.each_with_index.map do |line, i|
    has_nl = i < content_lines.length - 1 || last_has_nl
    [content_of(line), has_nl]
  end
  body =
    case style
    when 'literal' then literal_join(lines_with_nl)
    when 'folded' then folded_join(lines_with_nl)
    else raise "unknown style: #{style}"
    end
  apply_chomp(body, chomp)
end

def write_case(name, input, output)
  dir = File.join(SUITE_DIR, name)
  FileUtils.mkdir_p(dir)
  File.binwrite(File.join(dir, 'input.mical'), input)
  File.write(File.join(dir, 'output.json'), JSON.pretty_generate(output) + "\n")
end

def generate_case(style, explicit_indent, chomp, body_name, termination)
  body = BODIES.fetch(body_name)
  header = "key #{STYLES.fetch(style)}#{explicit_indent ? I_BASE : ''}#{CHOMPS.fetch(chomp)}"
  source_lines = [header] + render_body(body)

  case termination
  when 'eof'
    input = source_lines.join("\n") + "\n"
    last_has_nl = true
    extra = {}
  when 'eof-nonl'
    input = source_lines.join("\n")
    last_has_nl = false
    extra = {}
  when 'dedent'
    input = source_lines.join("\n") + "\nend marker\n"
    last_has_nl = true
    extra = { 'end' => 'marker' }
  else raise "unknown termination: #{termination}"
  end

  value = eval_block(style, chomp, body, last_has_nl)
  output = { 'key' => value }.merge(extra)

  indicator_part = explicit_indent ? "i#{I_BASE}-" : ''
  name = "block-string-gen-#{style}-#{indicator_part}#{chomp}-#{body_name}-#{termination}"
  write_case(name, input, output)
  name
end

# Regenerate from scratch.
Dir[File.join(SUITE_DIR, 'block-string-gen-*')].each { |d| FileUtils.rm_rf(d) }

generated = []

# Full style x chomp x termination cross for the simple body.
STYLES.each_key do |style|
  CHOMPS.each_key do |chomp|
    %w[eof eof-nonl dedent].each do |termination|
      generated << generate_case(style, false, chomp, 'simple', termination)
    end
  end
end

# Style x chomp for the structurally interesting bodies, dedent-terminated.
STYLES.each_key do |style|
  CHOMPS.each_key do |chomp|
    %w[extra-indent emptyln wsln trailing-empty].each do |body_name|
      generated << generate_case(style, false, chomp, body_name, 'dedent')
    end
  end
end

# Explicit indentation indicator: clip only, all bodies, dedent-terminated.
STYLES.each_key do |style|
  BODIES.each_key do |body_name|
    generated << generate_case(style, true, 'clip', body_name, 'dedent')
  end
end

# Tab content is literal in both styles regardless of termination.
STYLES.each_key do |style|
  generated << generate_case(style, false, 'clip', 'tab-content', 'eof')
end

puts "generated #{generated.length} cases"
