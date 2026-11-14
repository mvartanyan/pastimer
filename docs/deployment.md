# Deploying Pastimer on Horse / k3s

Status on 2026-11-14: deployed on Horse at https://1000.vartanyan.com, with [@pastimerbot](https://t.me/pastimerbot) connected through its default `Play / Играть` menu button. The [Horse deployment record](horse-table-cues-2026-11-14.md) contains the actual source revision, immutable image/digests, checks and backup paths. The canonical overlay is `deploy/horse/`; the rendered copy and operating notes are in the local `/Users/michael/dev/k3s-horse/pastimer` project. The owner confirmed real Telegram launch, signed-in identity and table creation. The current layout/computer release passes live solo and two-human browser games with signed synthetic identities; these are distinct from a manual real-client multiplayer playthrough.

The application runs as one replica with a persistent SQLite volume. The deployment uses Recreate updates so old and new pods do not write the database concurrently. The manifests create only the `pastimer` namespace and its application resources. Keep the cluster's existing ingress, certificate issuer and storage configuration.

## Actual Horse overlay

For this installation use `kubectl kustomize deploy/horse` and the scoped SSH commands in [the deployment record](horse-table-cues-2026-11-14.md). The base `deploy/k3s` directory remains hostname-neutral; the example ingress is still an example. Future releases must use a new tested immutable image and preserve the existing PVC. The generic instructions below also apply to a fresh installation.

## Container image

```sh
docker build -f deploy/Dockerfile -t pastimer:horse .
```

The image runs as UID/GID 10001, serves port 3000, enables production mode and writes SQLite under `/data`. It contains the server and offline verifier, embedded frontend assets and no development login. For managed Cloud builds using its inspecting proxy, supply the public combined trust bundle without adding it to the image:

```sh
docker build --secret id=proxy_ca,src=/etc/ssl/certs/ca-certificates.crt -f deploy/Dockerfile -t pastimer:horse .
```

Use an immutable image tag or digest for real updates. For a single Horse node, an operator with SSH/k3s access can transfer and import an image, without creating a registry:

```sh
docker save pastimer:horse | ssh horse 'sudo k3s ctr images import -'
```

For multiple nodes, use the cluster's existing registry or import the image on each eligible node. Set the Deployment image to that exact tag/digest and keep one SQLite writer. The PVC omits `storageClassName` to use the cluster default; confirm the bound volume and backup policy on Horse. If the cluster has no default, choose its established persistent storage class before applying.

## Connect the Telegram bot

1. Open **@BotFather** in Telegram. Create a bot with `/newbot`, or select an existing bot under `/mybots`. Keep the bot token private.
2. Supply that token to Horse as the Kubernetes Secret `pastimer-telegram`, key `bot-token`. Do not commit it. From a trusted administrator's Bash terminal, the following keeps it out of shell history and process arguments:

```sh
kubectl apply -f deploy/k3s/namespace.yaml
read -rsp 'Telegram bot token: ' PASTIMER_BOT_TOKEN
printf '\n'
printf '%s' "$PASTIMER_BOT_TOKEN" | kubectl -n pastimer create secret generic pastimer-telegram --from-file=bot-token=/dev/stdin --dry-run=client -o yaml | kubectl apply -f -
unset PASTIMER_BOT_TOKEN
```

The Secret must exist before the pod can start. The token must belong to the same bot whose Mini App users open; the server verifies Telegram initData with it. On later token changes, restart the Deployment because environment bindings are loaded on process startup.

3. Apply the application after setting the correct image:

```sh
kubectl apply -k deploy/k3s
kubectl -n pastimer rollout status deployment/pastimer --timeout=180s
kubectl -n pastimer get pods,pvc,service
```

4. Configure the public domain using Horse's existing ingress and certificate issuer. `deploy/k3s/ingress.example.yaml` shows the required service routing. Serve the frontend and API on the same **HTTPS origin**, preserve the Host header, and keep the database volume private. The HTTPS URL must serve Pastimer at `/`.
5. In **@BotFather → /mybots → your bot → Bot Settings → Menu Button**, choose a label such as `Play / Играть` and set the menu-button URL to that HTTPS origin. `/setmenubutton` is the equivalent BotFather command. This launches the Mini App and supplies signed user identity; a plain website link does not do that.
6. Open a private chat with your bot, press **Start**, then the menu button. Repeat from another Telegram account and test room creation, joining, bidding and a completed deal's audit export. Direct visitors outside Telegram cannot use local development login in production.

Pastimer does not consume Bot API updates, so it needs no webhook or polling worker. All game interaction happens in the Mini App. You can optionally configure a named Main Mini App in BotFather later; its short name and launch links are separate from the menu-button setup above.

## Readiness, updates and rollback

Check the actual public HTTPS `/api/health`, frontend assets and `/api/games`, then test Telegram login and play. A healthy pod alone does not validate the bot or TLS configuration.

Before updates, take a consistent SQLite backup (stop the Deployment first or use SQLite online backup). Keep the PVC when updating or rolling back. Update to an immutable tested image and wait for rollout; preserve the prior image for rollback. Avoid deleting the namespace/PVC as a deployment shortcut.
