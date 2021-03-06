# en-300-486-reader

Reader for DVB MPEG Transport Stream data formatted according to [ETSI standard EN 300 486](http://www.etsi.org/deliver/etsi_en/300400_300499/300468/01.15.01_60/en_300468v011501p.pdf),
"_Service Information in DVB systems_".

Based on the [`mpeg2ts-reader` crate](https://crates.io/crates/mpeg2ts-reader).

# Supported syntax

*Not much yet!*

 - Service Information table sections
   - [ ] NIT `network_information_section()`
   - [ ] BAT `bouquet_association_section()`
   - [x] SDT `service_description_section()`
   - [ ] EIT `event_information_section()`
   - [ ] TDT `time_date_section()`
   - [ ] TOT `time_offset_section()`
   - [ ] RST `running_status_section()`
   - [ ] ST `stuffing_section()`
   - [ ] DIT `discontinuity_information_section()`
   - [ ] SIT `selection_information_section()`
 - Descriptors
   - [ ] `network_name_descriptor`
   - [ ] `service_list_descriptor`
   - [ ] `stuffing_descriptor`
   - [ ] `satellite_delivery_system_descriptor`
   - [ ] `cable_delivery_system_descriptor`
   - [ ] `VBI_data_descriptor`
   - [ ] `VBI_teletext_descriptor`
   - [ ] `bouquet_name_descriptor`
   - [x] `service_descriptor`
   - [ ] `country_availability_descriptor`
   - [ ] `linkage_descriptor`
   - [ ] `NVOD_reference_descriptor`
   - [ ] `time_shifted_service_descriptor`
   - [ ] `short_event_descriptor`
   - [ ] `extended_event_descriptor`
   - [ ] `time_shifted_event_descriptor`
   - [ ] `component_descriptor`
   - [ ] `mosaic_descriptor`
   - [ ] `stream_identifier_descriptor`
   - [ ] `CA_identifier_descriptor`
   - [ ] `content_descriptor`
   - [ ] `parental_rating_descriptor`
   - [ ] `teletext_descriptor`
   - [ ] `telephone_descriptor`
   - [ ] `local_time_offset_descriptor`
   - [ ] `subtitling_descriptor`
   - [ ] `terrestrial_delivery_system_descriptor`
   - [ ] `multilingual_network_name_descriptor`
   - [ ] `multilingual_bouquet_name_descriptor`
   - [ ] `multilingual_service_name_descriptor`
   - [ ] `multilingual_component_descriptor`
   - [ ] `private_data_specifier_descriptor`
   - [ ] `service_move_descriptor`
   - [ ] `short_smoothing_buffer_descriptor`
   - [ ] `frequency_list_descriptor`
   - [ ] `partial_transport_stream_descriptor`
   - [ ] `data_broadcast_descriptor`
   - [ ] `scrambling_descriptor`
   - [ ] `data_broadcast_id_descriptor`
   - [ ] `transport_stream_descriptor`
   - [ ] `DSNG_descriptor`
   - [ ] `PDC_descriptor`
   - [ ] `AC3_descriptor`
   - [ ] `ancillary_data_descriptor`
   - [ ] `cell_list_descriptor`
   - [ ] `cell_frequency_link_descriptor`
   - [ ] `announcement_support_descriptor`
   - [ ] `application_signalling_descriptor`
   - [ ] `adaptation_field_data_descriptor`
   - [ ] `service_identifier_descriptor`
   - [ ] `service_availability_descriptor`
   - [ ] `default_authority_descriptor`
   - [ ] `related_content_descriptor`
   - [ ] `TVA_id_descriptor`
   - [ ] `content_identifier_descriptor`
   - [ ] `time_slice_fec_identifier_descriptor`
   - [ ] `ECM_repetition_rate_descriptor`
   - [ ] `S2_satellite_delivery_system_descriptor`
   - [ ] `enhanced_AC3_descriptor`
   - [ ] `DTS_descriptor`
   - [ ] `AAC_descriptor`
   - [ ] `XAIT_location_descriptor`
   - [ ] `FTA_content_management_descriptor`
   - [ ] `extension_descriptor`
