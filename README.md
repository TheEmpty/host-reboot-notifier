# Host Reboot Notifier
This is a rough application for sending prowl notifications
on host-reboots.

## Example Notification 
```
media-host: Restarted 
Media host was rebooted. Took 1 minute to come back up. Host was previously up for 48 hours.
```

## Example Docker Compose
```
  host-reboot-notifier:
    restart: unless-stopped
    image: theempty/host-reboot-notifier:latest
    container_name: host-reboot-notifier
    environment:
      - HOST_REBOOT_NOTIFIER_HOSTNAME=media-host
    command:
      - '/opt/host-reboot-notifier/config.json'
    volumes:
      - ./host-reboot-notifier:/opt/host-reboot-notifier
```

## Example Kube Setup

* DaemonSet so it runs on all hosts.
* `hostPath` so each node has its own data-store.

```
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: host-reboot-notifier
spec:
  selector:
    matchLabels:
      app: host-reboot-notifier
  template:
    metadata:
      labels:
        app: host-reboot-notifier
    spec:
      volumes:
        - name: config
          configMap:
            name: host-reboot-notifier-config
        - name: data
          hostPath:
            path: /var/host-reboot-notifier
      containers:
      - name: host-reboot-notifier
        image: theempty/host-reboot-notifier:latest
        resources:
          limits:
            memory: "128Mi"
            cpu: "1m"
        volumeMounts:
          - name: config
            mountPath: /etc/host-reboot-notifier
          - name: data
            mountPath: /var/host-reboot-notifier
        args:
          - /etc/host-reboot-notifier/config.json
        env:
          - name: RUST_LOG
            value: trace
          - name: HOST_REBOOT_NOTIFIER_HOSTNAME
            valueFrom:
              fieldRef:
                fieldPath: spec.nodeName

---

apiVersion: v1
kind: ConfigMap
metadata:
  name: host-reboot-notifier-config
data:
  config.json: |
    {
      "data_file": "/var/host-reboot-notifier/data.json",
      "prowl_api_keys": [
          "YOURS-HERE"
      ]
    }

```