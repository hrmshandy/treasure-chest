// NXM Protocol Types
export interface NxmUrl {
  game: string;
  mod_id: number;
  file_id: number;
  key: string;
  expires?: number;
  user_id?: number;
}

export interface NxmEvent {
  game: string;
  mod_id: number;
  file_id: number;
}
