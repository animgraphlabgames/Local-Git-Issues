export async function convertToWebp(file: File | Blob, quality = 0.8): Promise<Blob> {
  const bitmap = await createImageBitmap(file);
  const canvas = document.createElement('canvas');
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('Canvas 2D context not available');
  ctx.drawImage(bitmap, 0, 0);
  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob) resolve(blob);
        else reject(new Error('Failed to convert image to WebP'));
      },
      'image/webp',
      quality
    );
  });
}