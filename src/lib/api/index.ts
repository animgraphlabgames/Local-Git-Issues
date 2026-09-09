import { invoke } from '@tauri-apps/api/core';
import type { Issue, IssueEvent, IssueRevision, Label, Project, Dependency, CreateIssuePayload, UpdateIssueMetaPayload } from '$lib/types';

export const issuesApi = {
  getAll: () => invoke<Issue[]>('get_issues'),
  getEvents: (issueId: number) => invoke<IssueEvent[]>('get_issue_events', { issueId }),
  getRevisions: (issueId: number) => invoke<IssueRevision[]>('get_issue_revisions', { issueId }),
  create: (payload: CreateIssuePayload) =>
    invoke<Issue>('create_issue', {
      title: payload.title,
      body: payload.body,
      projectId: payload.projectId,
      labelIds: payload.labelIds
    }),
  updateContent: (id: number, title: string, body: string) =>
    invoke<Issue>('update_issue', { id, title, body }),
  updateMeta: (payload: UpdateIssueMetaPayload) =>
    invoke<Issue>('update_issue_meta', {
      id: payload.id,
      projectId: payload.projectId,
      labelIds: payload.labelIds
    }),
  toggleStatus: (id: number) => invoke<Issue>('toggle_issue_status', { id })
};

export const labelsApi = {
  getAll: () => invoke<Label[]>('get_labels'),
  create: (name: string, color: string, description: string) =>
    invoke<Label>('create_label', { name, color, description }),
  update: (id: number, name: string, color: string, description: string) =>
    invoke<Label>('update_label', { id, name, color, description }),
  delete: (id: number) => invoke<void>('delete_label', { id })
};

export const projectsApi = {
  getAll: () => invoke<Project[]>('get_projects'),
  create: (title: string, description: string) =>
    invoke<Project>('create_project', { title, description }),
  update: (id: number, title: string, description: string) =>
    invoke<Project>('update_project', { id, title, description }),
  delete: (id: number) => invoke<void>('delete_project', { id })
};

export const dependenciesApi = {
  getAll: () => invoke<Dependency[]>('get_dependencies'),
  checkAll: () => invoke<Dependency[]>('check_dependencies'),
  acknowledge: (id: number) => invoke<Dependency>('acknowledge_dependency', { id }),
  add: (name: string, target: string) => invoke<Dependency>('add_dependency', { name, target }),
  delete: (id: number) => invoke<void>('delete_dependency', { id })
};

export const systemApi = {
  getAutostart: () => invoke<boolean>('get_autostart'),
  setAutostart: (enabled: boolean) => invoke<void>('set_autostart', { enabled })
};

export const attachmentsApi = {
  save: (name: string, data: number[]) => invoke<string>('save_attachment', { name, data }),
  get: (name: string) => invoke<number[]>('get_attachment', { name })
};