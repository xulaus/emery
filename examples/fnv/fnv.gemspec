Gem::Specification.new do |s|
  s.name        = "fnv"
  s.version     = "0.1.0"
  s.summary     = "FNV Hash Function"
  s.description = "FNV extension to built in Digest Module"
  s.authors     = ["Richard Fitzgerald"]
  s.files = Dir['ext/fnv/src/*.rs', 'ext/fnv/Cargo.toml', 'ext/fnv/extconf.rb', 'tests/*.rb']
  s.extensions  = ['ext/fnv/extconf.rb']
end
