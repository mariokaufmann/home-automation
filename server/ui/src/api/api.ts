export async function get<T>(subUrl: string): Promise<T> {
  const url = `/api/${subUrl}`;
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error("Could not get data from API: " + response.statusText);
  }

  return await response.json();
}

export async function put<T>(subUrl: string, payload: T): Promise<void> {
  const url = `/api/${subUrl}`;
  const response = await fetch(url, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    throw new Error("Could not put data to API: " + response.statusText);
  }
}
