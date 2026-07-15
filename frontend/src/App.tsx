import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { notify } from "@tauri-apps/plugin-notification";

export default function App() {
  const [friendCode, setFriendCode] = useState("");
  const [community, setCommunity] = useState("team-abc");
  const [invite, setInvite] = useState("");
  const [verified, setVerified] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<string>("generate_friend_code")
      .then(setFriendCode)
      .catch(console.error);
  }, []);

  async function makeInvite() {
    const code = await invoke<string>("make_invite", {
      communityId: community,
      bootstrapPeers: ["peer1.onion"],
    });
    setInvite(code);
    setVerified(null);
    notify({ title: "Monika", body: "Invite link created" }).catch(() => {});
  }

  async function checkInvite() {
    if (!invite) return;
    const ok = await invoke<boolean>("verify_invite", { code: invite });
    setVerified(ok);
  }

  return (
    <main style={{ fontFamily: "system-ui, sans-serif", padding: 24 }}>
      <h1>Monika</h1>
      <p>
        Your friend code: <code>{friendCode || "…"}</code>
      </p>

      <section style={{ marginTop: 24 }}>
        <h2>Create an invite</h2>
        <input
          value={community}
          onChange={(e) => setCommunity(e.target.value)}
          placeholder="community id"
        />
        <button onClick={makeInvite} style={{ marginLeft: 8 }}>
          Create invite
        </button>
      </section>

      {invite && (
        <section style={{ marginTop: 24 }}>
          <h2>Invite link</h2>
          <code style={{ wordBreak: "break-all" }}>{invite}</code>
          <div>
            <button onClick={checkInvite} style={{ marginTop: 8 }}>
              Verify invite
            </button>
            {verified !== null && (
              <span style={{ marginLeft: 8 }}>
                {verified ? "valid ✓" : "invalid ✗"}
              </span>
            )}
          </div>
        </section>
      )}
    </main>
  );
}
