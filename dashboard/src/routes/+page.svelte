<script>
  import 'bulma/css/bulma.min.css';
  import { onMount } from 'svelte';
  import { HardDriveIcon, WindIcon, Trash2Icon, GithubIcon } from 'svelte-feather-icons';

  // @ts-ignore
  let usb = navigator.usb || window.usb;

  let port;
  let stats = [
    { laps: '0', last: '0.000', best: '0.000' },
    { laps: '0', last: '0.000', best: '0.000' },
    { laps: '0', last: '0.000', best: '0.000' }
  ];

  onMount(async () => {
    if (!usb) {
      return;
    }

    usb.addEventListener('connect', async ({ device }) => {
      open(device);
    });

    usb.addEventListener('disconnect', async ({ device }) => {
      if (port === device) {
        port = null;
      }
    });

    let knownDevices = await usb.getDevices();
    for (const device of knownDevices) {
      if (!device.opened) {
        await open(device);
        break;
      }
    }
  });

  async function toggleConnection() {
    if (port) {
      port.close();
      port = null;
    } else {
      let device = await usb.requestDevice({ filters: [{ vendorId: 0x16c0, productId: 0x27dd }] });
      if (!device.opened) {
        open(device);
      }
    }
  }

  async function reset() {
    await port?.controlTransferOut({
      requestType: 'vendor',
      recipient: 'device',
      request: 0x01,
      value: 0x00,
      index: 0x00
    });
  }

  async function open(device) {
    await device.open();
    await device.claimInterface(0);
    port = device;
    poll();
  }

  async function poll() {
    if (!port) {
      return;
    }
    try {
      let report = await port.transferIn(1, 8);
      let reportType = report.data.getUint8(0);
      switch (reportType) {
        case 0x01:
          let track = report.data.getUint8(1);
          stats[track].laps = report.data.getUint16(2);
          stats[track].last = (report.data.getUint16(4) / 1000).toFixed(3);
          stats[track].best = (report.data.getUint16(6) / 1000).toFixed(3);
          break;
        case 0xff:
          stats = [
            { laps: '0', last: '0.000', best: '0.000' },
            { laps: '0', last: '0.000', best: '0.000' },
            { laps: '0', last: '0.000', best: '0.000' }
          ];
          break;
      }
    } catch (error) {
      console.log('usb poll failed', error);
    }
    setTimeout(poll, 10);
  }
</script>

<div class="workspace">
  <nav class="navbar is-dark">
    <div class="navbar-brand">
      <div class="navbar-item">
        <div class="tag is-dark is-size-5 is-family-code has-text-weight-bold" class:is-warn={port}>
          <span class="icon is-small">
            <WindIcon />
          </span>
          &nbsp; Wzhooh
        </div>
      </div>
    </div>
    <div class="navbar-item toolbar">
      <div class="buttons">
        {#if port}
          <button class="button is-small is-danger" title="Reset" on:click={reset}>
            <span class="icon">
              <Trash2Icon />
            </span>
            <span>&nbsp; Reset</span>
          </button>
        {/if}
        {#if usb}
          <button
            class="button is-small"
            title="Connect"
            class:is-success={!port}
            class:is-warning={port}
            on:click={toggleConnection}
          >
            <span class="icon">
              <HardDriveIcon />
            </span>
            <span>
              &nbsp; {port ? 'Disconnect' : 'Connect'}
            </span>
          </button>
        {/if}
        <a class="button is-info is-small" href="https://github.com/dotcypress/wzhooh">
          <span class="icon">
            <GithubIcon />
          </span>
        </a>
      </div>
    </div>
  </nav>
  {#if usb}
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
  {:else}
    <section class="hero is-danger">
      <div class="hero-body">
        <p class="subtitle">
          The <a href="https://caniuse.com/web-usb">WebUSB API</a> is not supported by your browser
        </p>
      </div>
    </section>
  {/if}
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
  .workspace .navbar {
    display: flex;
  }
  .workspace .navbar .toolbar {
    display: flex;
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
