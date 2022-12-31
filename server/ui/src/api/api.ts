export async function get<T>(subUrl: string): Promise<T> {
  const url = `/api/${subUrl}`;

  const response = await fetch(url);

  checkForLoginRequests(response);
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

  checkForLoginRequests(response);
  if (!response.ok) {
    throw new Error("Could not put data to API: " + response.statusText);
  }
}

function checkForLoginRequests(response: Response): boolean {
  // in case we need to login again follow the redirect
  const loginHeader = response.headers.get("X-AUTOMATION-LOGIN");
  if (loginHeader) {
    window.location.href = loginHeader;
    return true;
  }
  return false;
}
