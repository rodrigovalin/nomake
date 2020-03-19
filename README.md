# nomake

`nomake` is a small CLI utility to start a Kubernetes cluster for testing
purposes, with a few convenience features. It supports configuring access to ECR
or to a local registry. `nomake` uses [Kind](https://kind.sigs.k8s.io/) to start
the local cluster.

## Usage

The simplest way of using `nomake` is to create a simple cluster.

``` sh
$ nomake create --name test
$ # and to configure kubectl
$ eval $(nomake config --name test)
$ kubectl get namespaces
$ nomake delete --name test
```

## Configuring access to ECR

`nomake` can configure access to a private ECR repo. It requires the
[ecr-login-helper](https://github.com/awslabs/amazon-ecr-credential-helper) to
be in your PATH.

``` sh
$ nomake create --name test --ecr xxx.ecr.region.amazonaws.com
$ eval $(nomake config --name test)
$ kubectl create deployment example --image xxx.ecr.region.amazonaws.com/xxx
```

## Configuring access to a local registry

`nomake` can use a local registry to speed up local development. To start the
local cluster follow the instructions
[here](https://kind.sigs.k8s.io/docs/user/local-registry/) and then:

``` sh
$ nomake create --name test --use-local-registry "kind-registry"
$ eval $(nomake config --name test)
$ kubectl create deployment example --image localhost:5000/xxx
```
