# sys

We maintain a carbon copy of Tracy's public part, so we can build it
as a static library and also bindgen the low-level bindings based on
its public header.

Instead of submodules or manual copying, we are using `git-subtree` to
do this. It worth revisiting this as local checkout and 1 copy command
might be easier.

We are getting Tracy's `master` for now and can later switch to stable
release tags in the future.

## Initial setup

Just do the following from the git repository root:

```sh
# adding tracy remote and checking out its master in a staging branch
$ git remote add -f tracy-upstream git@github.com:wolfpld/tracy.git
$ git checkout -b staging-tracy tracy-upstream/master

# split off a subdirectory 'public' from its master into a separate branch
$ git subtree split --squash -P public --annotate="Tracy: " --rejoin -b tracy-public

# checkout our main and add 'public' parts above to our 'sys/tracy'
$ git checkout main
$ git subtree add -P sys/tracy --squash tracy-public
```

## How to update

Just do the following from the git repository root:

```sh
# switch back to the tracy's master and update it
$ git checkout staging-tracy
$ git pull tracy-upstream master

# update the subdirectory branch with changes received above
$ git subtree split -P public --annotate="Tracy: " --rejoin -b tracy-public

# checkout our main and merge new 'public' parts to update our 'sys/tracy'
$ git subtree merge -P sys/tracy --squash tracy-public
```
