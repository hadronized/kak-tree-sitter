# Completions for kak-tree-sitter and ktsctl.

# List of all known languages by KTS.
export def "nu-complete ktsctl-available-langs" [] {
  ^ktsctl query -a | str trim | lines | skip 2 | split column '|' lang | get lang  
}

# The Kakoune tree-sitter client and server
export extern kak-tree-sitter [
  --kakoune (-k)         # Whether we start from Kakoune
  --init: string         # Initialize the current session by injecting some rc
  --server (-s)          # Start the server, if not already started
  --daemonize (-d)       # Try to daemonize, if not already done
  --client (-c): string  # Kakoune client to connect with, if any
  --request (-r): string # JSON-serialized request
  --with-highlighting    # Insert Kakoune code related to highlighting
  --with-text-objects    # Insert Kakoune code related to text-objects
  --verbose (-v)         # Verbosity (can be accumulated)
  --help (-h)            # Print help
  --version (-V)         # Print version
]

# The Kakoune tree-sitter controller
export extern ktsctl [
  --verbose (-v)         # Verbosity (can be accumulated)
  --help (-h)            # Print help
  --version (-V)         # Print version
]

# Fetch resources
export extern "ktsctl fetch" [
  lang?: string@"nu-complete ktsctl-available-langs" # Language to fetch
  --all (-a)                                         # Fetch all languages
  --help (-h)                                        # Print help
]

# Compile resources
export extern "ktsctl compile" [
  lang: string@"nu-complete ktsctl-available-langs" # Language to compile
  --all (-a)                                        # Compile all languages
  --help (-h)                                       # Print help
]

# Install resources
export extern "ktsctl install" [
  lang?: string@"nu-complete ktsctl-available-langs" # Language to install
  --all (-a)                                         # Install all languages
  --help (-h)                                        # Print help
]

# Synchronize resources
export extern "ktsctl sync" [
  lang?: string@"nu-complete ktsctl-available-langs" # Language to synchronize
  --all (-a)                                         # Synchronize all languages
  --help (-h)                                        # Print help
]

# Query resources
export extern "ktsctl query" [
  lang?: string@"nu-complete ktsctl-available-langs" # Language to query
  --all (-a)                                         # Query all languages
  --help (-h)                                        # Print help
]

# Remove resources (alias: rm)
export extern "ktsctl remove" [
  lang: string@"nu-complete ktsctl-available-langs" # Language to query
  --grammar (-g)                                    # Remove grammar
  --queries (-q)                                    # Remove queries
  --prune (-p)                                      # Prune resources
  --help (-h)                                       # Print help
]

# Print help
export extern "ktsctl help" []
