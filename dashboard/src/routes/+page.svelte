<script>
  import 'bulma/css/bulma.min.css';
  // @ts-ignore
  import { serial as polyfill } from 'web-serial-polyfill';
  import { HardDriveIcon, WindIcon, Trash2Icon, GithubIcon } from 'svelte-feather-icons';

  let port;
  let scratch = '';
  let decoder = new TextDecoder('utf-8');

  let version = '';
  let stats = [
    { laps: 0, last: '0', best: '0' },
    { laps: 0, last: '0', best: '0' },
    { laps: 0, last: '0', best: '0' }
  ];

  async function connect() {
    if (!port) {
      // @ts-ignore
      let serial = navigator.serial || polyfill;
      port = await serial?.requestPort({ filters: [{ usbVendorId: 0x16c0 }] });
      await port.open({ baudRate: 115200 });
      await send('version');
      poll();
    } else {
      version = '';
      await port.close();
      port = null;
    }
  }

  async function poll() {
    const reader = port?.readable?.getReader();
    if (!reader) {
      return;
    }

    try {
      const res = await reader.read();
      scratch += decoder.decode(res.value);
    } catch (error) {
      console.log(error);
    } finally {
      reader?.releaseLock();
    }

    let nl = scratch.indexOf('\n');
    while (nl != -1) {
      let line = scratch.substring(0, nl);
      scratch = scratch.substring(nl + 1);
      ingest(line);
      nl = scratch.indexOf('\n');
    }

    await send('stats');
    setTimeout(poll, 100);
  }

  async function send(command) {
    const writer = port?.writable?.getWriter();
    if (!writer) {
      return;
    }
    const encoder = new TextEncoder();
    await writer.write(encoder.encode(`${command}\r\n`));
    await writer.close();
  }

  async function reset() {
    await send('reset');
  }

  function ingest(line) {
    if (line.startsWith('wzhooh: ')) {
      version = line.substring(8);
    } else if (line.startsWith('track #')) {
      let track = parseInt(line.substring(7, 8));
      let tags = line.substring(8).split(';');
      for (const tag of tags) {
        let tagValue = tag.trim().split(':');
        if (tagValue.length == 2) {
          let value = parseInt(tagValue[1].trim()) || 0;
          switch (tagValue[0]) {
            case 'laps':
              stats[track].laps = value;
              break;
            case 'last':
              stats[track].last = (value / 1000000).toFixed(4);
              break;
            case 'best':
              stats[track].best = (value / 1000000).toFixed(4);
              break;
          }
        }
      }
    }
  }
</script>

<div class="workspace">
  <nav class="navbar is-dark">
    <div class="navbar-brand">
      <div class="navbar-item">
        <span
          class="tag is-dark is-size-5 is-family-code has-text-weight-bold"
          class:is-warn={port}
        >
          <span class="icon is-small">
            <WindIcon />
          </span>
          &nbsp; wzhooh
        </span>
      </div>
    </div>
    <div class="navbar-item toolbar">
      <div class="buttons">
        {#if version}
          <a
            class="button is-link is-small"
            href="https://github.com/dotcypress/wzhooh/releases/tag/{version}"
          >
            <span class="icon">
              <GithubIcon />
            </span>
            <span>&nbsp; Firmware {version}</span>
          </a>
        {/if}
        {#if port}
          <button class="button is-small is-danger" title="Reset" on:click={reset}>
            <span class="icon">
              <Trash2Icon />
            </span>
            <span>&nbsp; Reset</span>
          </button>
        {/if}
        <button
          class="button is-small"
          title="Connect"
          class:is-success={!port}
          class:is-warning={port}
          on:click={connect}
        >
          <span class="icon">
            <HardDriveIcon />
          </span>
          <span>
            &nbsp; {port ? 'Disconnect' : 'Connect'}
          </span>
        </button>
      </div>
    </div>
  </nav>
  <div class="columns">
    {#each stats as stat, i}
      <div class="column">
        <nav class="panel is-warning">
          <div class="panel-heading">
            <h1 class="title is-5">Track #{i + 1}</h1>
          </div>
          <div class="panel-block">
            <h1 class="title is-1 is-family-code has-text-weight-bold">{stat.laps}</h1>
            <div class="tag is-link is-light title is-6">LAPS</div>
          </div>
          <div class="panel-block">
            <h1 class="title is-2 is-family-code has-text-weight-bold">{stat.last}</h1>
            <div class="tag is-success is-light title is-6">LAST LAP</div>
          </div>
          <div class="panel-block">
            <h1 class="title is-2 is-family-code has-text-weight-bold">{stat.best}</h1>
            <div class="tag is-warning is-light title is-6">BEST LAP</div>
          </div>
        </nav>
      </div>
    {/each}
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    padding: 0;
    margin: 0;
    min-height: 100%;
  }
  .workspace {
    display: flex;
    flex-direction: column;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }
  .workspace .navbar .toolbar {
    flex: 1;
    justify-content: end;
  }
  .workspace .columns {
    margin: 8px;
    display: flex;
    flex: 1;
  }
  .column .panel-block {
    align-items: stretch;
    flex-direction: column;
    display: flex;
  }
  .column .panel-block h1 {
    padding-top: 1rem;
    text-align: center;
  }
  .column .panel-block .is-1 {
    font-size: 5.5rem;
  }
</style>
