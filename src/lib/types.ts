export interface Label {
  id: number;
  name: string;
  color: string;
  description: string;
}

export interface Project {
  id: number;
  title: string;
  description: string;
}

export type IssueStatus = 'open' | 'closed';

export interface Issue {
  id: number;
  title: string;
  body: string;
  status: IssueStatus;
  project_id: number | null;
  project_title: string | null;
  labels: Label[];
  created_at: string;
}

export interface IssueEvent {
  id: number;
  issue_id: number;
  event_type: 'status_change' | 'title_change' | 'label_added' | 'label_removed' | 'project_change' | 'cross_reference';
  old_value: string | null;
  new_value: string | null;
  metadata: string | null;
  created_at: string;
}

export interface IssueRevision {
  id: number;
  issue_id: number;
  body: string;
  created_at: string;
}

export interface CreateIssuePayload {
  title: string;
  body: string;
  projectId: number | null;
  labelIds: number[];
}

export interface UpdateIssueMetaPayload {
  id: number;
  projectId: number | null;
  labelIds: number[];
}

export type NavTab = 'issues' | 'labels' | 'projects';
export type IssueView = 'list' | 'detail' | 'new';