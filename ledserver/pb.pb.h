// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: pb.proto

#ifndef PROTOBUF_pb_2eproto__INCLUDED
#define PROTOBUF_pb_2eproto__INCLUDED

#include <string>

#include <google/protobuf/stubs/common.h>

#if GOOGLE_PROTOBUF_VERSION < 3000000
#error This file was generated by a newer version of protoc which is
#error incompatible with your Protocol Buffer headers.  Please update
#error your headers.
#endif
#if 3000000 < GOOGLE_PROTOBUF_MIN_PROTOC_VERSION
#error This file was generated by an older version of protoc which is
#error incompatible with your Protocol Buffer headers.  Please
#error regenerate this file with a newer version of protoc.
#endif

#include <google/protobuf/arena.h>
#include <google/protobuf/arenastring.h>
#include <google/protobuf/generated_message_util.h>
#include <google/protobuf/metadata.h>
#include <google/protobuf/message.h>
#include <google/protobuf/repeated_field.h>
#include <google/protobuf/extension_set.h>
#include <google/protobuf/unknown_field_set.h>
// @@protoc_insertion_point(includes)

// Internal implementation detail -- do not call these.
void protobuf_AddDesc_pb_2eproto();
void protobuf_AssignDesc_pb_2eproto();
void protobuf_ShutdownFile_pb_2eproto();

class LED;
class None;

// ===================================================================

class LED : public ::google::protobuf::Message /* @@protoc_insertion_point(class_definition:LED) */ {
 public:
  LED();
  virtual ~LED();

  LED(const LED& from);

  inline LED& operator=(const LED& from) {
    CopyFrom(from);
    return *this;
  }

  static const ::google::protobuf::Descriptor* descriptor();
  static const LED& default_instance();

  void Swap(LED* other);

  // implements Message ----------------------------------------------

  inline LED* New() const { return New(NULL); }

  LED* New(::google::protobuf::Arena* arena) const;
  void CopyFrom(const ::google::protobuf::Message& from);
  void MergeFrom(const ::google::protobuf::Message& from);
  void CopyFrom(const LED& from);
  void MergeFrom(const LED& from);
  void Clear();
  bool IsInitialized() const;

  int ByteSize() const;
  bool MergePartialFromCodedStream(
      ::google::protobuf::io::CodedInputStream* input);
  void SerializeWithCachedSizes(
      ::google::protobuf::io::CodedOutputStream* output) const;
  ::google::protobuf::uint8* InternalSerializeWithCachedSizesToArray(
      bool deterministic, ::google::protobuf::uint8* output) const;
  ::google::protobuf::uint8* SerializeWithCachedSizesToArray(::google::protobuf::uint8* output) const {
    return InternalSerializeWithCachedSizesToArray(false, output);
  }
  int GetCachedSize() const { return _cached_size_; }
  private:
  void SharedCtor();
  void SharedDtor();
  void SetCachedSize(int size) const;
  void InternalSwap(LED* other);
  private:
  inline ::google::protobuf::Arena* GetArenaNoVirtual() const {
    return _internal_metadata_.arena();
  }
  inline void* MaybeArenaPtr() const {
    return _internal_metadata_.raw_arena_ptr();
  }
  public:

  ::google::protobuf::Metadata GetMetadata() const;

  // nested types ----------------------------------------------------

  // accessors -------------------------------------------------------

  // optional uint32 index = 1;
  void clear_index();
  static const int kIndexFieldNumber = 1;
  ::google::protobuf::uint32 index() const;
  void set_index(::google::protobuf::uint32 value);

  // optional fixed32 rgb = 2;
  void clear_rgb();
  static const int kRgbFieldNumber = 2;
  ::google::protobuf::uint32 rgb() const;
  void set_rgb(::google::protobuf::uint32 value);

  // @@protoc_insertion_point(class_scope:LED)
 private:

  ::google::protobuf::internal::InternalMetadataWithArena _internal_metadata_;
  bool _is_default_instance_;
  ::google::protobuf::uint32 index_;
  ::google::protobuf::uint32 rgb_;
  mutable int _cached_size_;
  friend void  protobuf_AddDesc_pb_2eproto();
  friend void protobuf_AssignDesc_pb_2eproto();
  friend void protobuf_ShutdownFile_pb_2eproto();

  void InitAsDefaultInstance();
  static LED* default_instance_;
};
// -------------------------------------------------------------------

class None : public ::google::protobuf::Message /* @@protoc_insertion_point(class_definition:None) */ {
 public:
  None();
  virtual ~None();

  None(const None& from);

  inline None& operator=(const None& from) {
    CopyFrom(from);
    return *this;
  }

  static const ::google::protobuf::Descriptor* descriptor();
  static const None& default_instance();

  void Swap(None* other);

  // implements Message ----------------------------------------------

  inline None* New() const { return New(NULL); }

  None* New(::google::protobuf::Arena* arena) const;
  void CopyFrom(const ::google::protobuf::Message& from);
  void MergeFrom(const ::google::protobuf::Message& from);
  void CopyFrom(const None& from);
  void MergeFrom(const None& from);
  void Clear();
  bool IsInitialized() const;

  int ByteSize() const;
  bool MergePartialFromCodedStream(
      ::google::protobuf::io::CodedInputStream* input);
  void SerializeWithCachedSizes(
      ::google::protobuf::io::CodedOutputStream* output) const;
  ::google::protobuf::uint8* InternalSerializeWithCachedSizesToArray(
      bool deterministic, ::google::protobuf::uint8* output) const;
  ::google::protobuf::uint8* SerializeWithCachedSizesToArray(::google::protobuf::uint8* output) const {
    return InternalSerializeWithCachedSizesToArray(false, output);
  }
  int GetCachedSize() const { return _cached_size_; }
  private:
  void SharedCtor();
  void SharedDtor();
  void SetCachedSize(int size) const;
  void InternalSwap(None* other);
  private:
  inline ::google::protobuf::Arena* GetArenaNoVirtual() const {
    return _internal_metadata_.arena();
  }
  inline void* MaybeArenaPtr() const {
    return _internal_metadata_.raw_arena_ptr();
  }
  public:

  ::google::protobuf::Metadata GetMetadata() const;

  // nested types ----------------------------------------------------

  // accessors -------------------------------------------------------

  // @@protoc_insertion_point(class_scope:None)
 private:

  ::google::protobuf::internal::InternalMetadataWithArena _internal_metadata_;
  bool _is_default_instance_;
  mutable int _cached_size_;
  friend void  protobuf_AddDesc_pb_2eproto();
  friend void protobuf_AssignDesc_pb_2eproto();
  friend void protobuf_ShutdownFile_pb_2eproto();

  void InitAsDefaultInstance();
  static None* default_instance_;
};
// ===================================================================


// ===================================================================

#if !PROTOBUF_INLINE_NOT_IN_HEADERS
// LED

// optional uint32 index = 1;
inline void LED::clear_index() {
  index_ = 0u;
}
inline ::google::protobuf::uint32 LED::index() const {
  // @@protoc_insertion_point(field_get:LED.index)
  return index_;
}
inline void LED::set_index(::google::protobuf::uint32 value) {
  
  index_ = value;
  // @@protoc_insertion_point(field_set:LED.index)
}

// optional fixed32 rgb = 2;
inline void LED::clear_rgb() {
  rgb_ = 0u;
}
inline ::google::protobuf::uint32 LED::rgb() const {
  // @@protoc_insertion_point(field_get:LED.rgb)
  return rgb_;
}
inline void LED::set_rgb(::google::protobuf::uint32 value) {
  
  rgb_ = value;
  // @@protoc_insertion_point(field_set:LED.rgb)
}

// -------------------------------------------------------------------

// None

#endif  // !PROTOBUF_INLINE_NOT_IN_HEADERS
// -------------------------------------------------------------------


// @@protoc_insertion_point(namespace_scope)

// @@protoc_insertion_point(global_scope)

#endif  // PROTOBUF_pb_2eproto__INCLUDED
