{{/*
Resource naming: labhsathi-<service>-<kind>, per docs/CONVENTIONS.md.
*/}}
{{- define "labhsathi.name" -}}
{{- printf "labhsathi-%s-%s" .service .kind -}}
{{- end -}}

{{- define "labhsathi.labels" -}}
app.kubernetes.io/part-of: labhsathi
app.kubernetes.io/component: {{ .service }}
{{- end -}}
