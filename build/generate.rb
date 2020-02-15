#!/usr/bin/env ruby

require 'erb'
require 'ostruct'

# Generator for HTML.rs tags/attribute static metadata
class Generator

  attr_reader :tags, :attributes

  BASEDIR = File.dirname( __FILE__ )

  OUT_FILE  = File.join( BASEDIR, '../src/dom/html/meta.rs' )

  def run( out_file = OUT_FILE )
    parse_tags
    parse_attributes
    map_basic_attributes
    generate( out_file )
  end

  FLAGS = {
    'E' => 'empty',
    'D' => 'deprecated',
    'I' => 'inline',
    'M' => 'meta',
    'B' => 'banned',
    'U' => 'undefined'}

  def parse_tags
    @tags = []

    open( File.join( BASEDIR, 'tags' ), 'r' ) do |fin|
      fin.each do |line|
        case line
        when /^\s*#/, /^\s*$/
          # ignore comment, empty lines
        when /^\s*[^\s,]+\s*,[^,]*,[^,]*$/
          r = line.split(',').map { |c| c.strip }
          r = r.compact.reject { |c| c.empty? }
          flags = r[1].split(' ').map { |f| FLAGS[f] }.compact
          @tags << OpenStruct.new( :name => r[0],
                                   :flags => flags,
                                   :desc => r[2] )
        else
          raise "Parse ERROR: line [#{line}]"
        end
      end
    end

    @tags.sort_by! { |o| o.name }
    @tags.uniq! { |o| o.name }
    @tag_max_len = @tags.map { |t| t.name.length }.max
  end

  def parse_attributes
    @attributes = []
    tagsets = {}

    open( File.join( BASEDIR, 'attributes' ), 'r' ) do |fin|
      fin.each do |line|
        case line
        when /^\s*#/, /^\s*$/
          # ignore comment, empty lines
        when /^\s*([A-Z]+)\s*::\s*ALL\s+except:(.*)$/
          sname = $1
          except = $2.split( ' ' ).compact.reject { |t| t.empty? }
          tset = @tags.reject { |t| except.include?( t.name ) }
          tset.map! { |t| t.name }
          tagsets[sname] = tset
        when /^\s*[^\s,]+\s*,/
          r = line.split(',').map { |c| c.strip }
          r = r.compact.reject { |c| c.empty? }
          if r[3]
            flags = r[3].split(' ').map { |f| FLAGS[f] }.compact
          else
            flags = []
          end
          # FIXME: Handle attributes, desc.

          btags = r[1].split(' ').compact.reject { |t| t.empty? || t =~ /^\*/ }
          btags = btags.map { |t| tagsets[ t ] || t }.flatten

          @attributes << OpenStruct.new( :name => r[0],
                                         :basic_tags => btags,
                                         :flags => flags,
                                         :desc => r[2] )
        else
          raise "Parse ERROR: line [#{line}]"
        end
      end
    end

    @attributes.sort_by! { |o| o.name }
    @attributes.uniq! { |o| o.name }

    @attr_max_len = @attributes.map { |t| t.name.length }.max
  end

  def map_basic_attributes
    @tags.each do |tag|
      tag.basic_atts =
        @attributes.select { |attr| attr.basic_tags.include?( tag.name ) }
    end
  end

  def twidth( val, extra = 0 )
    val + ( ' ' * ( @tag_max_len - val.length + extra )  )
  end

  def awidth( val, extra = 0 )
    val + ( ' ' * ( @attr_max_len - val.length + extra )  )
  end

  def const( val )
    val.gsub( /\-/, '_' )
  end

  def clone_if( o, val )
    if o.flags.include?('undefined')
      "#{val}.clone()"
    else
      val
    end
  end

  def map_flags( tag )
    tag.flags
      .reject { |f| f == "undefined" }
      .map { |f| "is_#{f}: true" }
  end

  def generate( out_file )
    erb_file = File.join( BASEDIR, 'meta.rs.erb' )
    template = ERB.new( IO.read( erb_file ), nil, '%' )

    open( out_file, 'w' ) do |fout|
      fout << template.result( binding )
    end
  end

end

if $0 == __FILE__
  Generator.new.run( *ARGV )
end
