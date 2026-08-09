# Configuration

fjm comes with many features out of the box. Some of them are not activated by default as they’re changing your shell default behavior, and some are just a feature flag to avoid breaking changes or just experimental until we decide it is worthwhile to introduce them.

All these features can be configured by adding flags to the `fjm env` call when initializing the shell. For instance, if your shell set up looks like `eval "$(fjm env)"` then you can add a flag to it by changing it to `eval "$(fjm env --my-flag=value)"`

Here’s a list of these features and capabilities:

### `--use-on-cd`

**✅ Highly recommended**

`--use-on-cd` appends output to `fjm env`'s output that will hook into your shell upon changing directories, and will switch the JDK version based on the requirements of the current directory, based on a `.java-version` file.

This allows you do avoid thinking about `fjm use`, and only `cd <DIR>` to make it work.

### `--version-file-strategy=recursive`

**✅ Highly recommended**

Makes `fjm use` and `fjm install` take parent directories into account when looking for a version file ("dotfile")--when no argument was given.

So, let's say we have the following directory structure:

```
repo/
├── .java-version <- with content: `21`
└── packages/
  └── my-package/ <- I am here
```

And I'm running the following command:

```sh-session
repo/packages/my-package$ fjm use
```

Then fjm will switch to JDK 21.

Without the explicit flag, the value is set to `local`, which will not traverse the directory tree and therefore will print:

```sh-session
repo/packages/my-package$ fjm use
error: Can't find version in dotfiles. Please provide a version manually to the command.
```

### `--jdk-dist-mirror` / `FJM_JDK_DIST_MIRROR`

fjm resolves JDK versions and downloads against the [Adoptium (Eclipse Temurin) API](https://api.adoptium.net) by default. If you need to point fjm at a different Adoptium-compatible mirror (for example, a self-hosted or cached mirror), override it with the `--jdk-dist-mirror` flag or the `FJM_JDK_DIST_MIRROR` environment variable.

To list the versions available from an alternate mirror:

```sh-session
fjm --jdk-dist-mirror https://my-mirror.example.com ls-remote
```

To install and use a version from that mirror:

```sh-session
fjm --jdk-dist-mirror https://my-mirror.example.com use 21
```

Once installed, the version shows up like any other in your list:

```sh-session
fjm ls
```

And you can use it again without providing `--jdk-dist-mirror`:

```sh-session
fjm use 21
```

You can also set this permanently via the `FJM_JDK_DIST_MIRROR` environment variable instead of passing the flag on every invocation.
