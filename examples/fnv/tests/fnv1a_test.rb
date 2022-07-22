require "fnv"
require 'minitest/autorun'

describe EMERY::FNV1a do
  describe "whens given non string data" do
    it "raises an error" do
      assert_raises(RuntimeError){ EMERY::FNV1a.hexdigest(nil) }
    end
  end

  describe "whens given symbol data" do
    it "raises an error" do
      assert_raises(RuntimeError){ EMERY::FNV1a.hexdigest(:adsadsa) }
    end
  end

  describe "whens given string data" do
    it "returns expected hash" do
      assert_equal 3290367854661108436, EMERY::FNV1a.hexdigest("adsadsa")
    end
  end
end
