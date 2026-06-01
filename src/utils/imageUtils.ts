// src/utils/imageUtils.ts

/**
 * Gets the base URL for the backend API, dynamically adjusting 
 * if accessed via localhost or across a local network IP.
 */
export const getApiBaseUrl = (): string => {
  const host = window.location.hostname === 'localhost' 
    ? 'localhost' 
    : window.location.hostname;
  return `http://${host}:3000`;
};

/**
 * GET: Constructs a full, network-accessible URL for displaying an image.
 * Use this in your Vue templates to bind image sources.
 * * @param path - The relative path from the database (e.g., /uploads/123.png)
 * @returns The absolute network URL
 */
export const getImageUrl = (path: string | null | undefined): string => {
  if (!path) return '';
  return `${getApiBaseUrl()}${path}`;
};

/**
 * POST: Uploads an image file to the Rust backend.
 * * @param file - The image File object from an input element
 * @returns A promise that resolves to the saved file path
 */
export const uploadImage = async (file: File): Promise<string> => {
  const ext = file.name.split('.').pop() || 'png';
  
  const res = await fetch(`${getApiBaseUrl()}/api/inventory/upload-photo`, {
    method: 'POST',
    headers: {
      'x-file-ext': ext,
      'Content-Type': 'application/octet-stream'
    },
    body: file
  });
  
  if (!res.ok) throw new Error('Failed to upload photo');
  return await res.json();
};