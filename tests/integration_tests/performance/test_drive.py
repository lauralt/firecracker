# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests for drive performance with fio tool."""

import os
import json
import pytest

import host_tools.drive as drive_tools
import host_tools.network as net_tools   # pylint: disable=import-error

MIN_VALUES = {'bw': 150000, 'iops': 40000}
BS_VALUES = ['4k', '32k', '128k', '512k', '1M']
IO_DEPTHS = ['1', '2', '8', '32']
IO_PATTERNS = ['randread', 'randwrite', 'read', 'write']


@pytest.mark.timeout(0)
def test_drive_performance(test_microvm_with_ssh, network_config):
    """Test the output of fio for read / write operations on /dev/vdb"""
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

    fio_config = 'integration_tests/performance/config.fio'
    fio_config_remote = 'config.fio'
    ssh_connection.scp_file(fio_config, fio_config_remote)
    print('\n')
    for io_pattern in IO_PATTERNS:
        for block_size in BS_VALUES:
            for io_depth in IO_DEPTHS:
                _, stdout, stderr = ssh_connection.execute_command("IO_PATTERN=" + io_pattern + " BLOCK_SIZE=" +
                                                                   block_size + " IO_DEPTH=" + io_depth +
                                                                   " fio config.fio --output-format=json")
                assert stderr.read().decode('utf-8') == ''
                print(json.loads(stdout.read().decode('utf-8')))


# def test_standard_read_drive_performance(test_microvm_with_ssh, network_config):
#     """Test the output of fio for io read operations on /dev/vdb.
#     A job file with standard values for the parameters is passed to the fio command.
#     """
#     test_microvm = test_microvm_with_ssh
#     test_microvm.spawn()
#
#     test_microvm.basic_config(vcpu_count=1, ht_enabled=False, mem_size_mib=512)
#     _tap, _, _ = test_microvm.ssh_network_config(network_config, '1')
#
#     fs1 = drive_tools.FilesystemFile(
#         os.path.join(test_microvm.fsfiles, 'scratch'), size=2048
#     )
#     response = test_microvm.drive.put(
#         drive_id='scratch',
#         path_on_host=test_microvm.create_jailed_resource(fs1.path),
#         is_root_device=False,
#         is_read_only=False
#     )
#     assert test_microvm.api_session.is_status_no_content(response.status_code)
#
#     test_microvm.start()
#     ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)
#
#     fio_config = 'integration_tests/performance/config.fio'
#     fio_config_remote = 'config.fio'
#     ssh_connection.scp_file(fio_config, fio_config_remote)
#     _, stdout, stderr = ssh_connection.execute_command("fio config.fio --output-format=json")
#
#     assert stderr.read().decode('utf-8') == ''
#
#     fio_results = json.loads(stdout.read().decode('utf-8'))
#     job_results = fio_results["jobs"]
#     read_stats = job_results[0]["read"]
#
#     for fio_param in MIN_VALUES:
#         if fio_param in read_stats:
#             assert read_stats[fio_param] > MIN_VALUES[fio_param]
#
#
# def test_standard_write_drive_performance(test_microvm_with_ssh, network_config):
#     """Test the output of fio for io write operations on /dev/vdb.
#     A job file with standard values for the parameters is passed to the fio command.
#     """
#     test_microvm = test_microvm_with_ssh
#     test_microvm.spawn()
#
#     test_microvm.basic_config(vcpu_count=1, ht_enabled=False, mem_size_mib=512)
#     _tap, _, _ = test_microvm.ssh_network_config(network_config, '1')
#
#     fs1 = drive_tools.FilesystemFile(
#         os.path.join(test_microvm.fsfiles, 'scratch'), size=2048
#     )
#     response = test_microvm.drive.put(
#         drive_id='scratch',
#         path_on_host=test_microvm.create_jailed_resource(fs1.path),
#         is_root_device=False,
#         is_read_only=False
#     )
#     assert test_microvm.api_session.is_status_no_content(response.status_code)
#
#     test_microvm.start()
#     ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)
#
#     fio_config = 'integration_tests/performance/config_fio_write'
#     fio_config_remote = 'config_fio_write'
#     ssh_connection.scp_file(fio_config, fio_config_remote)
#     _, stdout, stderr = ssh_connection.execute_command("fio config_fio_write --output-format=json")
#
#     assert stderr.read().decode('utf-8') == ''
#
#     fio_results = json.loads(stdout.read().decode('utf-8'))
#     job_results = fio_results["jobs"]
#     write_stats = job_results[0]["write"]
#
#     for fio_param in MIN_VALUES:
#         if fio_param in write_stats:
#             assert write_stats[fio_param] > MIN_VALUES[fio_param]

# @pytest.mark.timeout(0)
# def test_drive_performance(test_microvm_with_ssh, network_config):
#     """Test the output of fio for read / write operations on /dev/vdb"""
#     test_microvm = test_microvm_with_ssh
#     test_microvm.spawn()
#
#     test_microvm.basic_config(vcpu_count=1, ht_enabled=False, mem_size_mib=512)
#     _tap, _, _ = test_microvm.ssh_network_config(network_config, '1')
#
#     fs1 = drive_tools.FilesystemFile(
#         os.path.join(test_microvm.fsfiles, 'scratch'), size=2048
#     )
#     response = test_microvm.drive.put(
#         drive_id='scratch',
#         path_on_host=test_microvm.create_jailed_resource(fs1.path),
#         is_root_device=False,
#         is_read_only=False
#     )
#
#     assert test_microvm.api_session.is_status_no_content(response.status_code)
#
#     test_microvm.start()
#     ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)
#
#     print('\n')
#     for io_pattern in IO_PATTERNS:
#         for block_size in BS_VALUES:
#             for io_depth in IO_DEPTHS:
#                 _, stdout, stderr = ssh_connection.execute_command("fio --name=fio_" + io_pattern + " --rw=" +
#                                                                    io_pattern + " --bs=" + block_size +
#                                                                    " --ioengine=libaio --iodepth=" +
#                                                                    str(io_depth) + " --filename=/dev/vdb" +
#                                                                    " --size=2G --direct=1 --randrepeat=0" +
#                                                                    " --loops=10 --output-format=json")
