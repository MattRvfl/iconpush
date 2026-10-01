// Builds an iOS configuration profile (.mobileconfig) containing one Web Clip per icon.
// Format reference: https://developer.apple.com/documentation/devicemanagement/webclip

const xmlEscape = (s) => String(s).replace(/[<>&'"]/g, (c) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', "'": '&apos;', '"': '&quot;' }[c]));

const uuid = () => (crypto.randomUUID ? crypto.randomUUID() : 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
  const r = (Math.random() * 16) | 0;
  return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16);
})).toUpperCase();

// U+2800 (braille blank) renders as an empty label on the home screen.
export const BLANK_LABEL = '⠀';

/**
 * @param {{name: string, clips: {label: string, url: string, png: string}[]}} profile
 *   png is a data URL (data:image/png;base64,...)
 * @returns {string} the .mobileconfig XML
 */
export function buildMobileconfig(profile) {
  const clips = profile.clips.map((c) => {
    const id = uuid();
    const base64 = c.png.replace(/^data:image\/png;base64,/, '');
    return `
		<dict>
			<key>FullScreen</key>
			<false/>
			<key>Icon</key>
			<data>${base64}</data>
			<key>IsRemovable</key>
			<true/>
			<key>Label</key>
			<string>${xmlEscape(c.label)}</string>
			<key>PayloadDisplayName</key>
			<string>${xmlEscape(c.label.trim() || c.url)}</string>
			<key>PayloadIdentifier</key>
			<string>dev.iconpush.clip.${id}</string>
			<key>PayloadType</key>
			<string>com.apple.webClip.managed</string>
			<key>PayloadUUID</key>
			<string>${id}</string>
			<key>PayloadVersion</key>
			<integer>1</integer>
			<key>Precomposed</key>
			<true/>
			<key>URL</key>
			<string>${xmlEscape(c.url)}</string>
		</dict>`;
  }).join('');

  const rootId = uuid();
  return `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>PayloadContent</key>
	<array>${clips}
	</array>
	<key>PayloadDescription</key>
	<string>Icônes personnalisées créées avec iconpush (https://github.com/MattRvfl/iconpush).</string>
	<key>PayloadDisplayName</key>
	<string>${xmlEscape(profile.name)}</string>
	<key>PayloadIdentifier</key>
	<string>dev.iconpush.${rootId}</string>
	<key>PayloadOrganization</key>
	<string>iconpush</string>
	<key>PayloadRemovalDisallowed</key>
	<false/>
	<key>PayloadType</key>
	<string>Configuration</string>
	<key>PayloadUUID</key>
	<string>${rootId}</string>
	<key>PayloadVersion</key>
	<integer>1</integer>
</dict>
</plist>
`;
}
