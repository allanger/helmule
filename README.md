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
  - name: my-git-mirror
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
  - name: my-git-mirror
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

To mirror a chart you'll have to define repos first. You can choose between `helm` and `git` repos.

```yaml
repositories:
  - name: db-operator
    git:
      url: https://github.com/db-operator/charts.git
      ref: main
      path: charts
  - name: flux-community
    helm:
      url: https://fluxcd-community.github.io/helm-charts
  - name: bitnami-oci
    helm:
      # it can be OCI too
      url: oci://registry-1.docker.io/bitnamicharts/

```

And then you need to start adding charts

```yaml
charts:
    # it should be the real chart name
  - name: flux2
    # the name of the repo from the .repositories
    repository: flux-community
    # it will be latest if not provided, or you can fix it here
    #
    # but if you use git repos, it makes sense to always use latest
    # becase in a git repo usually there is only one version
    # of a chart, and you either need to guess it, or just fix it
    # on the repo level, and use latest here
    version: latest
    mirrors:
        - custom-command
        - my-git-mirror

```

Let's assume you've named this file `helmule.yaml`, and now it's time to test it. Execute the following
```bash
helmule -c helmule.yaml -d -w test
```

Also, `variables` can be defined per chart with `charts.[].variables` then they going to be merged with global ones

It's going to create the `test` folder, where you'll be able to find both target repos and charts to be mirrored. If you're happy with the result, run it without `-d` and `-w test` to actually mirror.

Until this moment, it looks exactly like a yet another helm chart mirroring solution, right? At least, it does to me. So let's walk through features.

---

First one is `extensions` config property.
> Currently it will only cope files to your chart after it was pulled from the upstream. But later I plan to add an ability to use variables there, so one extension can more-or-less easily be reused between charts.

Why would one need it? Well, me for example, in my clusters I use Istio as an ingress controller and db-operator to manage databases, and since usually charts are provided with `ingress` and obviously without `database` custom resources, I can't just use them without modifications. I don't want to modify manifests that are generated by helm, because of many reasons. So I need to have them in the helm chart. Since I'm using helmfile, I can add them as dependencies without forking charts, but it still not the perfect solution I guess.

So, here come extensions. You can create a `VirtualService` and a `Database`, let's say you've created something like that:
```
helmule.yaml
extensions
  _helper.tpl
  database.yaml
  virtual-service.yaml
```

And then you can add it to you helmule config

```yaml
charts:
  - name: flux2
    repository: flux-community
    version: latest
    # Obviously, flux doesn't need any of those extensions,
    # but it's just for a sake of example
    extensions:
        - name: Create a VirtualService and a Database
          # Relative to the config
          source_dir: ./extensions
          # Relative to the chart root
          target_dir: ./templates/extensions
    mirrors:
        - custom-command
        - my-git-mirror
```

And if you run helmule now, you'll see additional resources added to the chart.

> Extensions currently are always trying to create the dir that is defined as a `target_dir`, so if it exists, extension will fail. It will be fixed later, but currently that's what we've got

---

The second feature is `patches`. Those are supposed to be modifications to helm chart that require more logic than just copy-paste.

There are 4 kinds of patches at the moment.

1. Regexp patch. This one will create a regexp and try to replace it with desired value. It's not checking if anything was actually changed though. It can be defined like that
```yaml
charts:
  - name: flux2
    patches:
      - name: Some regexp patch
        regexp:
          path: ./patch/to/patch
```

And the patch itself should also be a yaml file:

```yaml
---
name: Replace image with picture
targets:
  - values.yaml
before: image
after: picture
```

It was the first kind of patch added, so it's more or less POC. I wouldn't recommend using it at this point, unless you're really sure what you are doing

2. Git patch. You see "Git" here, but it doesn't mean that it's working only with git repos. What it's doing is it's initializing a git repo in the chart dir, and trying to apply a git patch you've provided. To create one, you could use a flag `--init-git-patch $CHART_NAME_1,$CHART_NAME_2`. Then helmule won't try to mirror charts that are listed under that flug, but it will init git repos in their root folders and prints paths to them, so you can go there, change something, execute `git diff > $CONFIG_DIR/chart.patch` and then add it to the config like that:

```yaml
charts:
  - name: flux2
    patches:
      - name: Some git patch
        git:
          path: ./flux2.patch
```

3. Yq patch. It's just a helper for yq.
```yaml
charts:
  - name: flux2
    patches:
      - name: yq example
        yq:
          op: Add # or Replace, or Remove
          key: .home # Just a yaml key (also can be .annotations."something.annotation")
          value: localhost # A value to set, is not required if operation is Remove
          files: Chart.yaml # A file that should be changed
```

4. CustomCommand patch. It's not really a patch, I'd say it can be anything you want. It will just execute commands from the root dir if a chart. Example:
```yaml
  - name: yamlfmt
    custom_command:
      commands:
        - "cat <<EOT >> .yamlfmt\n  formatter:\n    pad_line_comments: 2\nEOT"
        - yamlfmt values.yaml --conf ./yamlfmt.yaml
        - rm -f yamlfmt.yaml
```

It's also possible to re-use patches by different charts. If you have a common patch, you can define it on the root level of you config:
```yaml
charts: {}
patches:
  - name: yamlfmt
    custom_command:
      commands:
        - "cat <<EOT >> .yamlfmt\n  formatter:\n    pad_line_comments: 2\nEOT"
        - yamlfmt values.yaml --conf ./yamlfmt.yaml
        - rm -f yamlfmt.yaml
```

And then, if you define a patch with the name only for a chart, it's going to be taken from global patches

```yaml
charts:
  - name: flux2
    patches:
      - name: yamlfmt
patches:
  - name: yamlfmt
    custom_command:
      commands:
        - "cat <<EOT >> .yamlfmt\n  formatter:\n    pad_line_comments: 2\nEOT"
        - yamlfmt values.yaml --conf ./yamlfmt.yaml
        - rm -f yamlfmt.yaml
```

---

One last thing that is not described yet is the `include` property. It looks like that:
```yaml
include:
  - kind Charts # Can also be Repositories and Mirrors, other properties will be includable later
    path: ./charts.yaml
charts: {}
```

The included file will be marshalled into a vector of desired property. But the file that is included can contain either a list or just a single object, so both will work

```yaml
# charts.yaml
name: flux2
repository: flux-community
mirrors:
  - my-git-mirror
```

```yaml
# charts.yaml
- name: flux2
  repository: flux-community
  mirrors:
    - my-git-mirror
```
