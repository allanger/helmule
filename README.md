Helmule

This tool is not production ready yet, I'm still changing the config format, so don't rely on it for a time being.

# What it's supposed to do?

It would be just a yet another tool to mirror helm charts, but there is a couple of features that (I hope) are making this tool special. So let's go through all of them.

So, let's imaging you need to mirror a helm chart for whatever reason. Maybe you just don't trust original authors that much, or you use ArgoCD ~~~that doesn't know what is helm and how it's supposed to be used~~~, or whatever else. We'll start by a simple mirroring and then walk through all features later.

First we create a config file

```yaml
repositories: {}
charts: {}
mirrors: {}
```

Currently there are two types of mirrors that are supported:
- Git 
- Custom Comand


Let's start with git. 

```yaml
# Basic example
mirrors:
  - name: my-git-mirror
    git:
      url: git@git.badhouseplants.net:allanger/helmuled-charts.git
      branch: mirror-something
      path: charts/something
      commit: |-
        chore: mirror something
        
```

As you can see, it won't work on scale, so all the field can be templated using the chart data and a couple of helpers.

```yaml
  - name: badhouseplants-git
    git:
      url: git@git.badhouseplants.net:allanger/helmuled-charts.git
      branch: upgrade-{{ name }}-to-{{ version }}
      path: charts/{{ name }}
      commit: |-
        chore: mirror {{ name }}-{{ version }}

        upstream_repo: {{ repo_url }}
```

It can be scaled better already. URL can also be templated, and there is special property for variables, that you can also use here

```yaml
variables:
  git-msg: Hello there

mirrors:
  - name: badhouseplants-git
    git:
      url: git@git.badhouseplants.net:allanger/helmuled-charts.git
      branch: upgrade-{{ name }}-to-{{ version }}
      path: charts/{{ name }}
      commit: |-
        chore: mirror {{ name }}-{{ version }}
        {{ vars.git-msg }}
        upstream_repo: {{ repo_url }}
```

Currently, there are two available helpers:
- date: `{{ date }}`
- time: `{{ time }}`

Also you can provide `rebase<bool>` and `default_branch<string>`, if you want helmiule to rebase before pushing.

That would be it for git, and now the second option: CustomCommand

The process of mirroring is split into two parts:
- Package 
- Upload

The second is being executed only if you don't run in the `dry-run` mode. Git mirror handles it in code. But for custom command you'll have to define it yourself. Just check the following example:

```yaml
mirrors:
  - name: custom-command
    custom_command:
      package:
        - zip -r {{ name }}-{{ version }}.zip {{ name }}-{{ version }}
      upload:
        - rm -f /tmp/{{ name }}-{{ version }}.zip
        - rm -rf /tmp/{{ name }}-{{ version }}
        - cp {{ name }}-{{ version }}.zip /tmp
        - unzip /tmp/{{ name }}-{{ version }}.zip -d /tmp/{{ name }}-{{ version}}
```

These command are executed from the workdir. It's created during the run, and by default it's using a library to create a temporary directory, but you also can chose one by providing the `-w/--workdir` flag. Run will fail if this folder exists though, because it's expected to be created by helmule.

Now, when we got our mirrors, we need to start mirroring.

...to be continued...


