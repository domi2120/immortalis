
{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "immortalis.fullname" -}}
{{- if contains .Chart.Name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "immortalis.fullnameapi" -}}
{{ include "immortalis.fullname" . }}-api
{{- end }}

{{- define "immortalis.fullnamearchiver" -}}
{{ include "immortalis.fullname" . }}-archiver
{{- end }}

{{- define "immortalis.fullnametracker" -}}
{{ include "immortalis.fullname" . }}-tracker
{{- end }}

{{- define "immortalis.fullnamemigrator" -}}
{{ include "immortalis.fullname" . }}-migrator
{{- end }}

{{- define "immortalis.fullnameclient" -}}
{{ include "immortalis.fullname" . }}-client
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "immortalis.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "immortalis.labels" -}}
helm.sh/chart: {{ include "immortalis.chart" . }}
{{ include "immortalis.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "immortalis.selectorLabels" -}}
app.kubernetes.io/name: {{ include "immortalis.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Api Selector labels
*/}}
{{- define "immortalis.apiSelectorLabels" -}}
app.kubernetes.io/name: {{ include "immortalis.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: api
{{- end }}

{{/*
Archiver Selector labels
*/}}
{{- define "immortalis.archiverSelectorLabels" -}}
app.kubernetes.io/name: {{ include "immortalis.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: archiver
{{- end }}

{{/*
Archiver Selector labels
*/}}
{{- define "immortalis.trackerSelectorLabels" -}}
app.kubernetes.io/name: {{ include "immortalis.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: tracker
{{- end }}

{{/*
Client Selector labels
*/}}
{{- define "immortalis.clientSelectorLabels" -}}
app.kubernetes.io/name: {{ include "immortalis.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: client
{{- end }}