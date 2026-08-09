# `fjm`

```
A fast and simple JDK manager

Usage: fjm [OPTIONS] <COMMAND>

Commands:
  list-remote  List all remote JDK versions [alias: ls-remote]
  list         List all locally installed JDK versions [alias: ls]
  install      Install a new JDK version [alias: i]
  use          Change JDK version
  env          Print and set up required environment variables for fjm
  completions  Print shell completions to stdout
  alias        Alias a version to a common name
  unalias      Remove an alias definition
  default      Set a version as the default version or get the current default version
  current      Print the current JDK version
  exec         Run a command within fjm context
  uninstall    Uninstall a JDK version [alias: uni]
  help         Print this message or the help of the given subcommand(s)

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

# `fjm list-remote`

```
List all remote JDK versions

Usage: fjm list-remote [OPTIONS]

Options:
      --filter <FILTER>
          Filter versions by a user-defined version or a semver range

      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --lts
          Show only LTS versions

      --sort <SORT>
          Version sorting order

          Possible values:
          - desc: Sort versions in descending order (latest to earliest)
          - asc:  Sort versions in ascending order (earliest to latest)

          [default: asc]

      --latest
          Only show the latest matching version

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm list`

```
List all locally installed JDK versions

Usage: fjm list [OPTIONS]

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm install`

```
Install a new JDK version

Usage: fjm install [OPTIONS] [VERSION]

Arguments:
  [VERSION]
          A version string. Can be a partial semver or a LTS version name by the format lts/NAME

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --lts
          Install latest LTS

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --latest
          Install latest version

      --progress <PROGRESS>
          Show an interactive progress bar for the download status

          [default: auto]
          [possible values: auto, never, always]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --use
          Use the installed version immediately after installation

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm use`

```
Change JDK version

Usage: fjm use [OPTIONS] [VERSION]

Arguments:
  [VERSION]


Options:
      --install-if-missing
          Install the version if it isn't installed yet

      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --silent-if-unchanged
          Don't output a message identifying the version being used if it will not change due to execution of this command

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm env`

```
Print and set up required environment variables for fjm

This command generates a series of shell commands that should be evaluated by your shell to create a fjm-ready environment.

Each shell has its own syntax of evaluating a dynamic expression. For example, evaluating fjm on Bash and Zsh would look like `eval "$(fjm env --shell bash)"`. In Fish, evaluating would look like `fjm env --shell fish | source`

Usage: fjm env [OPTIONS]

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --shell <SHELL>
          The shell syntax to use. Infers when missing

          [possible values: bash, zsh, fish, powershell]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --json
          Print JSON instead of shell commands

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --use-on-cd
          Print the script to change JDK versions every directory change

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm completions`

```
Print shell completions to stdout

Usage: fjm completions [OPTIONS]

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --shell <SHELL>
          The shell syntax to use. Infers when missing

          [possible values: bash, zsh, fish, powershell]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm alias`

```
Alias a version to a common name

Usage: fjm alias [OPTIONS] <TO_VERSION> <NAME>

Arguments:
  <TO_VERSION>


  <NAME>


Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm unalias`

```
Remove an alias definition

Usage: fjm unalias [OPTIONS] <REQUESTED_ALIAS>

Arguments:
  <REQUESTED_ALIAS>


Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm default`

```
Set a version as the default version or get the current default version.

This is a shorthand for `fjm alias VERSION default`

Usage: fjm default [OPTIONS] [VERSION]

Arguments:
  [VERSION]


Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm current`

```
Print the current JDK version

Usage: fjm current [OPTIONS]

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm exec`

```
Run a command within fjm context

Example:
--------
fjm exec --using=17.0.2 java --version
=> 17.0.2

Usage: fjm exec [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...
          The command to run

Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --using <VERSION>
          Either an explicit version, or a filename with the version written in it

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm uninstall`

```
Uninstall a JDK version

> Warning: when providing an alias, it will remove the JDK version the alias > is pointing to, along with the other aliases that point to the same version.

Usage: fjm uninstall [OPTIONS] [VERSION]

Arguments:
  [VERSION]


Options:
      --jdk-dist-mirror <JDK_DIST_MIRROR>
          Adoptium (Eclipse Temurin) API base URL override

          [env: FJM_JDK_DIST_MIRROR]
          [default: https://api.adoptium.net]

      --fjm-dir <BASE_DIR>
          The root directory of fjm installations

          [env: FJM_DIR]

      --log-level <LOG_LEVEL>
          The log level of fjm commands

          [env: FJM_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed JDK binary. Defaults to arch of fjm binary

          [env: FJM_ARCH]
          [possible values: x86, x64, x64-musl, x64-glibc217, arm64, armv7l, ppc64le, ppc64, s390x]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is called without a version, or when `--use-on-cd` is configured on evaluation

          Possible values:
          - local:     Use the local version of the JDK defined within the current directory
          - recursive: Use the version of the JDK defined within the current directory and all parent directories

          [env: FJM_VERSION_FILE_STRATEGY]
          [default: local]

  -h, --help
          Print help (see a summary with '-h')
```

# `fjm help`

```

```
