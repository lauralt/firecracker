# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests for drive performance with fio tool."""

import os
import json

import host_tools.drive as drive_tools
import host_tools.network as net_tools   # pylint: disable=import-error


MAX_VALUES = {'bw': 400000, 'iops': 100000, 'runtime': 3000}
MIN_VALUES = {'bw': 150000, 'iops': 40000, 'runtime': 1400}


def test_read_drive_performance(test_microvm_with_ssh, network_config):
    """Test the output of fio for read operations on /dev/vdb"""
    test_microvm = test_microvm_with_ssh
    test_microvm.spawn()

    test_microvm.basic_config(vcpu_count=1, ht_enabled=False, mem_size_mib=512)
    _tap, _, _ = test_microvm.ssh_network_config(network_config, '1')

    fs1 = drive_tools.FilesystemFile(
        os.path.join(test_microvm.fsfiles, 'scratch'), size=2048
    )
    response = test_microvm.drive.put(
        drive_id='scratch',
        path_on_host=test_microvm.create_jailed_resource(fs1.path),
        is_root_device=False,
        is_read_only=False
    )

    assert test_microvm.api_session.is_status_no_content(response.status_code)

    test_microvm.start()
    ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)

    fio_config = 'integration_tests/performance/config_fio_read'
    fio_config_remote = 'config_fio_read'
    ssh_connection.scp_file(fio_config, fio_config_remote)
    _, stdout, stderr = ssh_connection.execute_command("fio config_fio_read --output-format=json")
    assert stderr.read().decode('utf-8') == ''

    fio_results = json.loads(stdout.read().decode('utf-8'))
    job_results = fio_results["jobs"]
    read_stats = job_results[0]["read"]

    for fio_param in MAX_VALUES:
        if fio_param in read_stats:
            assert read_stats[fio_param] < MAX_VALUES[fio_param]
            assert read_stats[fio_param] > MIN_VALUES[fio_param]


def test_write_drive_performance(test_microvm_with_ssh, network_config):
    """Test the output of fio for write operations on /dev/vdb"""
    test_microvm = test_microvm_with_ssh
    test_microvm.spawn()

    test_microvm.basic_config(vcpu_count=1, ht_enabled=False, mem_size_mib=512)
    _tap, _, _ = test_microvm.ssh_network_config(network_config, '1')

    fs1 = drive_tools.FilesystemFile(
        os.path.join(test_microvm.fsfiles, 'scratch'), size=2048
    )
    response = test_microvm.drive.put(
        drive_id='scratch',
        path_on_host=test_microvm.create_jailed_resource(fs1.path),
        is_root_device=False,
        is_read_only=False
    )

    assert test_microvm.api_session.is_status_no_content(response.status_code)

    test_microvm.start()
    ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)

    fio_config = 'integration_tests/performance/config_fio_write'
    fio_config_remote = 'config_fio_write'
    ssh_connection.scp_file(fio_config, fio_config_remote)
    _, stdout, stderr = ssh_connection.execute_command("fio config_fio_write --output-format=json")
    assert stderr.read().decode('utf-8') == ''

    fio_results = json.loads(stdout.read().decode('utf-8'))
    job_results = fio_results["jobs"]
    write_stats = job_results[0]["write"]

    for fio_param in MAX_VALUES:
        if fio_param in write_stats:
            assert write_stats[fio_param] < MAX_VALUES[fio_param]
            assert write_stats[fio_param] > MIN_VALUES[fio_param]
