interface IconProps {
  className?: string;
  size?: number;
}

function icon(size: number, props: IconProps) {
  return { width: props.size ?? size, height: props.size ?? size };
}

// ─── Navigation Icons ───

export function GlobeIcon(props: IconProps) {
  return (
    <svg {...icon(18, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
    </svg>
  );
}

export function SettingsIcon(props: IconProps) {
  return (
    <svg {...icon(18, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

export function ListIcon(props: IconProps) {
  return (
    <svg {...icon(18, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14,2 14,8 20,8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <polyline points="10,9 9,9 8,9" />
    </svg>
  );
}

export function InfoIcon(props: IconProps) {
  return (
    <svg {...icon(18, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="16" x2="12" y2="12" />
      <line x1="12" y1="8" x2="12.01" y2="8" />
    </svg>
  );
}

// ─── Action Icons ───

export function PlusIcon(props: IconProps) {
  return (
    <svg {...icon(15, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  );
}

export function ExternalLinkIcon(props: IconProps) {
  return (
    <svg {...icon(15, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15,3 21,3 21,9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </svg>
  );
}

export function CopyIcon(props: IconProps) {
  return (
    <svg {...icon(15, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

export function EditIcon(props: IconProps) {
  return (
    <svg {...icon(15, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
    </svg>
  );
}

export function TrashIcon(props: IconProps) {
  return (
    <svg {...icon(14, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="3,6 5,6 21,6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}

export function MonitorIcon(props: IconProps) {
  return (
    <svg {...icon(20, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="3" width="20" height="14" rx="2" />
      <line x1="8" y1="21" x2="16" y2="21" />
      <line x1="12" y1="17" x2="12" y2="21" />
    </svg>
  );
}

export function XamppIcon(props: IconProps) {
  return (
    <svg {...icon(18, props)} className={`xampp-icon ${props.className || ""}`} viewBox="0 0 24 24" fill="currentColor" stroke="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M16.792,11.923c0.113,0.043,0.226,0.079,0.334,0.128c0.45,0.203,0.715,0.553,0.748,1.044 c0.041,0.634,0.044,1.271,0.002,1.905c-0.049,0.732-0.725,1.292-1.483,1.271c-0.735-0.021-1.369-0.62-1.397-1.341 c-0.017-0.441-0.003-0.884-0.006-1.326c-0.001-0.239-0.003-0.242-0.245-0.243c-1.363-0.001-2.726,0.008-4.089-0.003 c-0.888-0.007-1.421,0.482-1.471,1.46c-0.019,0.38-0.1,0.727-0.357,1.018c-0.397,0.451-0.898,0.601-1.472,0.466 c-0.554-0.131-0.867-0.522-1.035-1.048c-0.117-0.367-0.056-0.737,0.012-1.094c0.341-1.797,1.366-3.006,3.125-3.555 c0.357-0.112,0.731-0.166,1.105-0.166c0.94,0.001,1.881,0.001,2.821-0.001c0.128,0,0.257-0.012,0.385-0.021 c0.702-0.051,1.166-0.511,1.22-1.352c0.004-0.064,0-0.129,0.001-0.193c0.011-0.788,0.605-1.396,1.393-1.425 c0.787-0.029,1.438,0.527,1.493,1.318c0.076,1.083-0.265,2.046-0.913,2.907C16.903,11.751,16.819,11.816,16.792,11.923z M8.249,10.436c-0.258-0.008-0.571,0.018-0.882-0.035c-0.536-0.09-0.876-0.39-1.02-0.916C6.19,8.912,6.25,8.388,6.698,7.96 C7.154,7.526,7.694,7.4,8.285,7.645c0.52,0.216,0.859,0.731,0.89,1.293C9.2,9.382,9.178,9.828,9.182,10.272 c0.001,0.116-0.043,0.167-0.161,0.165C8.781,10.434,8.542,10.436,8.249,10.436z M21.682,0H2.318C1.102,0,0.116,0.986,0.116,2.202 v19.317c0,1.37,1.111,2.481,2.481,2.481h18.807c1.37,0,2.481-1.111,2.481-2.481V2.202C23.884,0.986,22.898,0,21.682,0z M20.125,12.473c0.519,0.804,0.733,1.69,0.677,2.657c-0.108,1.886-1.413,3.474-3.25,3.916c-2.585,0.623-4.566-0.923-5.233-2.794 c-0.109-0.304-0.16-0.622-0.224-0.985c-0.068,0.414-0.115,0.789-0.264,1.134c-0.697,1.617-1.884,2.603-3.665,2.799 c-2.104,0.232-4.048-1.067-4.632-3.084c-0.25-0.863-0.175-1.747-0.068-2.625c0.08-0.653,0.321-1.268,0.632-1.848 c0.057-0.106,0.057-0.184-0.01-0.285c-0.561-0.845-0.779-1.777-0.7-2.784C3.43,8.035,3.56,7.52,3.805,7.038 C4.52,5.626,6.09,4.427,8.193,4.626c1.849,0.175,3.562,1.77,3.83,3.564c0.013,0.09,0.039,0.178,0.068,0.311 c0.044-0.241,0.076-0.439,0.118-0.636c0.344-1.63,1.94-3.335,4.201-3.357c2.292-0.021,3.99,1.776,4.31,3.446 c0.17,0.888,0.089,1.776-0.103,2.663c-0.112,0.517-0.31,1.008-0.524,1.492C20.034,12.245,20.043,12.345,20.125,12.473z"/>
    </svg>
  );
}

export function FilterIcon(props: IconProps) {
  return (
    <svg {...icon(14, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
    </svg>
  );
}

export function SearchIcon(props: IconProps) {
  return (
    <svg {...icon(15, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
  );
}

// ─── Sidebar Icons ───

export function ChevronLeftIcon(props: IconProps) {
  return (
    <svg {...icon(16, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="15,18 9,12 15,6" />
    </svg>
  );
}

export function ChevronRightIcon(props: IconProps) {
  return (
    <svg {...icon(16, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="9,18 15,12 9,6" />
    </svg>
  );
}

export function SunIcon(props: IconProps) {
  return (
    <svg {...icon(16, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="5" />
      <line x1="12" y1="1" x2="12" y2="3" />
      <line x1="12" y1="21" x2="12" y2="23" />
      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
      <line x1="1" y1="12" x2="3" y2="12" />
      <line x1="21" y1="12" x2="23" y2="12" />
      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
    </svg>
  );
}

export function MoonIcon(props: IconProps) {
  return (
    <svg {...icon(16, props)} className={props.className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}
