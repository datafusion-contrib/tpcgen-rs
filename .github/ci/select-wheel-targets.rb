# frozen_string_literal: true

require "json"
require "yaml"

catalog_path = File.join(__dir__, "wheel-targets.yml")
catalog = YAML.safe_load(File.read(catalog_path))
requested_names = ENV.fetch("WHEEL_TARGETS").split(",", -1).map(&:strip)
selected_names = requested_names == ["all"] ? catalog.keys : requested_names.uniq
invalid_names = selected_names.select { |name| name.empty? || !catalog.key?(name) }

if selected_names.empty? || invalid_names.any?
  abort "::error::Invalid wheel targets #{requested_names.inspect}. Use 'all' or: #{catalog.keys.join(', ')}"
end

matrix = selected_names.map { |name| catalog.fetch(name) }
puts "matrix=#{JSON.generate(include: matrix)}"
